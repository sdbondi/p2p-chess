use anyhow::anyhow;
use std::error::Error;

use crate::palette::Palette;
use crate::sprite::GetRgba;
use crate::Colour;
use crate::Frame;

/// `@struct` Bitmap
///
/// Representa uma imagem de bitmap.
pub struct Bitmap {
    /// `@private`
    /// Largura desta imagem (mantenha valores positivos e múltiplos de 2).
    width: i32,

    /// `@private`
    /// Altura desta imagem (mantenha valores positivos e múltiplos de 2).
    height: i32,

    /// `@private`
    /// Metadados desta imagem (opcional).
    metadata: u32,

    /// `@private`
    /// Paleta de cores desta imagem (até 256 cores).
    palette: Palette,

    /// `@private`
    /// Índice de cor usado para o canal de transparência desta imagem. Se esta
    // imagem for desenhada em cima de outra, este índice não será desenhado.
    alpha: u8,

    /// `@private`
    /// Quando este modo está ativado, todo pixel ou desenho que exceder os
    /// limites horizontais da tela é desenhado na extremidade oposta.
    x_wrapping: bool,

    /// `@private`
    /// Quando este modo está ativado, todo pixel ou desenho que exceder os
    /// limites verticais da tela é desenhado na extremidade oposta.
    y_wrapping: bool,

    /// `@private`
    /// Dados da imagem. É uma grade 2D de pixels, mas mantida como uma única
    // array contínua para economizar espaço.
    data: Vec<u8>,
}

impl Bitmap {
    /// `@constructor`
    ///
    /// # Parâmetro(s):
    ///
    /// `width` - Largura desta imagem (mantenha valores positivos e múltiplos de 2).
    /// `height` - Altura desta imagem (mantenha valores positivos e múltiplos de 2).
    /// `palette` - Paleta de cores desta imagem (até 256 cores).
    pub fn new(mut width: i32, mut height: i32, palette: Palette) -> Self {
        // Arredondadar dimensões de tamanho para valores positivos:
        width = width.abs();
        height = height.abs();

        // Arredondar dimensões de tamanho para múltiplos de 2. Valores ímpares
        // tendem a dar vários problemas com esse tipo de formato: a largura de
        // um bitmap, por exemplo, nãop ode possuir tamanho ímpar.
        width += width % 2;
        height += height % 2;

        Bitmap {
            width,
            height,
            metadata: 0,
            palette,
            alpha: 0,
            x_wrapping: false,
            y_wrapping: false,
            data: vec![0; (width * height) as usize],
        }
    }

    /// Cria um objeto de imagem a partir de um arquivo de bitmap.
    ///
    /// # Parâmetro(s):
    ///
    /// `content` - Conteúdo binário do arquivo.
    ///
    /// # Retorna:
    ///
    /// Um objeto de imagem, em caso de êxito, ou uma mensagem de erro, caso
    // ocorra alguma falha.
    pub fn from_bytes(content: &[u8]) -> Result<Bitmap, anyhow::Error> {
        // Tamanho do conteúdo, em bytes.
        let content_length: u32 = content.len() as u32;

        // @error
        //
        // Um arquivo de bitmap possui um cabeçalho de, no mínimo, 14 bytes. Se
        // o conteúdo passado for menor que isso, a operação será encerrada:
        if content_length < 0x0E {
            return Err(anyhow!(
                "Content length is too short to even contain a header (minimum 14 bytes)."
            ));
        }

        // Assinatura do conteúdo, vulgo, "número mágico".
        let header_signature: String = String::from_utf8_lossy(&content[..2]).to_string();

        // @error
        //
        // Um arquivo de bitmap sempre começa com as iniciais "BM". Se o
        // conteúdo passado não começar com elas, a operação será encerrada:
        if header_signature != "BM" {
            return Err(anyhow!(
                "Content is not a bitmap file (they start with \"BM\")."
            ));
        }

        // Tamanho total do conteúdo, em bytes. A ordenação dos bytes está em
        // little-endian, ou seja, devem ser lidos da direita para a esquerda.
        let header_file_size: u32 =
            u32::from_le_bytes([content[0x02], content[0x03], content[0x04], content[0x05]]);

        // @error
        //
        // Um arquivo de bitmap descreve seu tamanho exato (em bytes). Se o
        // conteúdo não possuir o mesmo tamanho, a operação será encerrada:
        if header_file_size != content_length {
            return Err(anyhow!("Content length doesn't match with header."));
        }

        // Metadados da imagem. Estes são na verdade 2 valores de 16 bits, mas
        // seu uso é reservado e varia entre as aplicações.
        let header_metadata: u32 =
            u32::from_be_bytes([content[0x06], content[0x07], content[0x08], content[0x09]]);

        // Offset de onde os dados da imagem começam.
        let header_offset: u32 =
            u32::from_le_bytes([content[0x0A], content[0x0B], content[0x0C], content[0x0D]]);

        // @error
        //
        // Por razões de simplicidade, esta biblioteca só utilizará bitmaps com
        // 256 colores.
        //
        // Curiosamente, todos os bitmaps com 256 colores indexadas começam, no
        // mesmo offset: o valor 0x436!
        //
        // Sendo assim, este será o valor usado como referência. Caso o offset
        // do conteúdo seja diferente, a operação, infelizmente, será encerrada:
        // if header_offset != 0x436 {
        //     return Err(anyhow!("Only 256-color bitmaps are supported."));
        // }

        // Dimensões de largura e altura da imagem.
        let dib_bitmap_width: u32 =
            u32::from_le_bytes([content[0x12], content[0x13], content[0x14], content[0x15]]);
        let dib_bitmap_height: u32 =
            u32::from_le_bytes([content[0x16], content[0x17], content[0x18], content[0x19]]);

        // @error
        //
        // Embora um arquivo de bitmap possa ter dimensões ímpares, o método
        // usado é relativamente confuso.
        //
        // Imagine uma imagem de tamanho 19x19: embora os metadados da imagem
        // indiquem 19 pixels de largura, na prática os dados são 20x19! A
        // largura é arredondado para cima, mas a altura permanece a mesma. Nos
        // editores de imagem, os "pixels fantasmas" são apenas omitidos, mas
        // permanecem lá.
        //
        // Por isso, para evitar problemas com esse tipo de formato, apenas
        // imagens com dimensões pares serão aceitas. Do contrário, a operação,
        // infelizmente, será encerrada:
        if dib_bitmap_width % 2 == 1 || dib_bitmap_height % 2 == 1 {
            return Err(anyhow!("Only power of 2 bitmaps are supported."));
        }

        // Número de bits por pixel (256 cores equivale a 8bpp).
        let dib_color_bpp: u16 = u16::from_le_bytes([content[0x1C], content[0x1D]]);

        // @error
        //
        // Por razões de simplicidade, esta biblioteca só utilizará bitmaps com
        // 8bpp. Caso seja diferente, a operação, infelizmente, será encerrada:
        if dib_color_bpp != 8 {
            return Err(anyhow!("Only 8bpp bitmaps are supported."));
        }

        // Método de compressão usado para o bitmap.
        let dib_compression: u32 =
            u32::from_le_bytes([content[0x1E], content[0x1F], content[0x20], content[0x21]]);

        // @error
        //
        // Por razões de simplicidade, esta biblioteca só utilizará bitmaps sem
        // compressão. Do contrário, a operação, infelizmente, será encerrada:
        if dib_compression != 0 {
            return Err(anyhow!("Only uncompressed bitmaps are supported."));
        }

        // Paleta de cores da imagem.
        let mut palette: Vec<Colour> = Vec::with_capacity(256);

        // Dados da imagem, ou seja, uma grade 2D de pixels.
        let data = &content[(header_offset as usize..content_length as usize)];

        // Percorrer conteúdo em torno da paleta de cores...
        for i in 0..256 {
            let idx = 0x36 + (i * 4);
            palette.push(Colour::new(
                content[idx + 2],
                content[idx + 1],
                content[idx],
                content[idx + 3],
            ));
        }

        // Objeto de imagem a ser retornado.
        let mut image: Bitmap = Bitmap::new(
            dib_bitmap_width as i32,
            dib_bitmap_height as i32,
            Palette::new(palette),
        );

        // Inserir dados restantes no objeto de imagem...
        image.metadata = header_metadata;
        image.data = data.to_vec();

        // Como os dados estão em little-endian, a array deverá ser invertida
        // para a orientação adequada (verticalmente):
        image.flip();

        Ok(image)
    }

    /// Exporta esta imagem para um arquivo de bitmap.
    ///
    /// # Retorna:
    ///
    /// Uma array de bytes contendo o arquivo de bitmap exportado.
    pub fn export_file(&mut self) -> Result<Vec<u8>, anyhow::Error> {
        // @error
        //
        // Os atributos da imagem devem estar todos alinhados corretamente.
        // Se as dimensões forem menores que zero, a operação será encerrada:
        if self.width < 1 || self.height < 1 {
            return Err(anyhow!(
                "Bitmap size (width and height) must be at least greater than zero."
            ));
        }

        // @error
        //
        // A imagem deve possuir pelo menos um pixel nos dados de imagem. Do
        // contrário, a operação será encerrada:
        if self.data.is_empty() {
            return Err(anyhow!("Bitmap image data must have at least 1 pixel."));
        }

        // Offset de onde os dados da imagem começam.
        //
        //Para simplificar a leitura/escrita, o formato é sempre um bitmap de
        // 256 cores e sem compressão. Sendo assim, o tamanho total do início
        // do arquivo pode ser calculado facilmente.
        let offset: usize = 0x436;

        // Tamanho total do conteúdo, em bytes.
        let file_size: u32 = (offset + self.data.len()) as u32;
        let file_size_bytes: Vec<u8> = file_size.to_le_bytes().to_vec();

        // Dimensões gerais do bitmap.
        let width_bytes: Vec<u8> = self.width.to_le_bytes().to_vec();
        let height_bytes: Vec<u8> = self.height.to_le_bytes().to_vec();
        let size_bytes: Vec<u8> = (self.width * self.height).to_le_bytes().to_vec();

        // Metadados do bitmap.
        let metadata_bytes: Vec<u8> = self.metadata.to_be_bytes().to_vec();

        // Conteúdo do arquivo de bitmap.
        let mut content: Vec<u8> = vec![
            // Assinatura do conteúdo ("BM"), vulgo, "número mágico".
            0x42,
            0x4D,
            // @template [u32; 0x02; little-endian]
            // Tamanho total do conteúdo, em bytes.
            file_size_bytes[0],
            file_size_bytes[1],
            file_size_bytes[2],
            file_size_bytes[3],
            // @template [u32; 0x03; big-endian]
            // Metadados da imagem.
            metadata_bytes[0],
            metadata_bytes[1],
            metadata_bytes[2],
            metadata_bytes[3],
            // Offset de onde os dados da imagem começam.
            //
            //Para simplificar a leitura/escrita, o formato é sempre um bitmap de
            // 256 cores e sem compressão. Sendo assim, o tamanho total do início
            // do arquivo pode ser calculado facilmente.
            0x36,
            0x04,
            0x00,
            0x00,
            // Tamanho do cabeçalho do conteúdo (40 bytes).
            0x28,
            0x00,
            0x00,
            0x00,
            // @template [u32; 0x12; little-endian]
            // Largura da imagem.
            width_bytes[0],
            width_bytes[1],
            width_bytes[2],
            width_bytes[3],
            // @template [u32; 0x14; little-endian]
            // Altura da imagem.
            height_bytes[0],
            height_bytes[1],
            height_bytes[2],
            height_bytes[3],
            // Número de planos de cor (deve ser sempre 1).
            0x01,
            0x00,
            // Número de bits por pixel (256 cores equivale a 8bpp).
            0x08,
            0x00,
            // Método de compressão usado para o bitmap.
            0x00,
            0x00,
            0x00,
            0x00,
            // @template [u32; 0x22; little-endian]
            // Tamanho total da imagem (width * height).
            size_bytes[0],
            size_bytes[1],
            size_bytes[2],
            size_bytes[3],
            // Tamanho da resolução horizontal desta imagem.
            0x12,
            0x0B,
            0x00,
            0x00,
            // Tamanho da resolução vertical desta imagem.
            0x12,
            0x0B,
            0x00,
            0x00,
            // Número de cores presentes na paleta.
            0x00,
            0x01,
            0x00,
            0x00,
            // Número de cores importantes usadas (pode ser ignorado).
            0x00,
            0x01,
            0x00,
            0x00,
        ];

        // Cor padrão usada quando não há um índice válido na paleta de cores.
        // Seu uso será visto logo abaixo...
        let default_color: Colour = Colour::new(0, 0, 0, 0);

        // @template [u8[256 * 4]; little-endian]
        // Paleta de cores da imagem.
        for i in 0..256 {
            // Obter referência da array padrão:
            let mut color: &Colour = &default_color;

            // Se existir um índice válido de paleta na imagem, esta
            // substituirá a referência declarada acima:
            if self.palette.len() > i {
                color = self.palette.get_color(i);
            }

            // Inserir cor na paleta...
            content.push(color.b);
            content.push(color.g);
            content.push(color.r);
            content.push(color.a);
        }

        // Inverter imagem. O bitmap guarda as imagens assim. A imagem será
        // restaurada depois:
        self.flip();

        // Inserir dados de imagem...
        for i in 0..self.data.len() {
            content.push(self.data[i]);
        }

        // Restaurar imagem para a sua orientação original:
        self.flip();

        Ok(content)
    }

    /// Exporta esta imagem para ser usado em um framebuffer.
    ///
    /// # Retorna:
    ///
    /// Uma array de bytes similar aos dados de imagem, mas com todos os
    /// atributos de cor diretamente convertidos para 32-bits em formato RGB.
    pub fn export_framebuffer(&self) -> Vec<u32> {
        // Conteúdo do framebuffer.
        let mut content: Vec<u32> = Vec::new();

        // Converter e adicionar pixels no framebuffer...
        for i in 0..self.data.len() {
            let pixel = self.data[i];
            let color: &Colour = self.palette.get_color(pixel as usize);
            content.push(color.to_rgba());
        }

        content
    }

    /// Escreve os dados de imagem diretamente em um framebuffer, isto é, uma
    /// array de 32-bits.
    ///
    /// # Parâmetro(s):
    ///
    /// `buffer` - Framebuffer.
    pub fn to_framebuffer(&self, buffer: &mut Vec<u32>) {
        // Converter e escrever pixels na referência do framebuffer...
        for i in 0..self.data.len() {
            let pixel = self.data[i];
            let color: &Colour = self.palette.get_color(pixel as usize);
            buffer[i] = color.to_rgba();
        }
    }

    /// Inverte esta imagem horizontalmente.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn mirror(&mut self) -> bool {
        // Array que conterá os dados espelhados.
        let mut mirrored_data: Vec<u8> = Vec::new();

        // Percorrer e salvar os pixels ao contrário...
        for y in 0..self.height {
            for x in (0..self.width).rev() {
                mirrored_data.push(self.get_pixel(x, y));
            }
        }

        self.data = mirrored_data;
        true
    }

    /// Inverte esta imagem verticalmente.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn flip(&mut self) -> bool {
        // Array que conterá os dados invertidos.
        let mut flipped_data: Vec<u8> = Vec::new();

        // Percorrer e salvar os pixels do avesso...
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                flipped_data.push(self.get_pixel(x, y));
            }
        }

        self.data = flipped_data;
        true
    }

    /// Rotaciona esta imagem em ângulos múltiplos de 90º.
    ///
    /// # Parâmetro(s):
    ///
    /// `angle` - Ângulo de rotação (positivos para sentido horário, negativos para sentido anti-horário).
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn rotate(&mut self, mut angle: i32) -> bool {
        // Ajustar o ângulo em múltiplos de 90º. Valores positivos e negativos
        // determinam se o sentido da rotação será horário ou anti-horário:
        angle = ((angle / 90) * 90) % 360;

        // Se não houver um ângulo válido, nada será feito:
        if angle == 0 {
            return false;
        }
        // Converter ângulo negativo em um equivalente positivo:
        else if angle < 0 {
            angle += 360;
        }

        //  O ângulo de 180º pode ser feito apenas invertendo a imagem:
        if angle == 180 {
            self.mirror();
            self.flip();

            return true;
        }

        // Array que conterá os dados invertidos + cache de tamanho.
        let mut rotated_data: Vec<u8> = Vec::new();
        let width = self.width;

        // Percorrer e salvar os pixels em iteração trocada...
        for x in 0..self.width {
            for y in (0..self.height).rev() {
                rotated_data.push(self.get_pixel(x, y));
            }
        }

        // Inverter dimensões da imagem e atualizar dados de imagem:
        self.width = self.height;
        self.height = width;
        self.data = rotated_data;

        //  O ângulo de 270º exige uma inversão de imagem logo após a rotação:
        if angle == 270 {
            self.mirror();
            self.flip();
        }

        true
    }

    /// Verifica se uma coordenada de pixel está visível na tela.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X.
    /// `y` - Posição Y.
    ///
    /// # Retorna:
    ///
    /// Resultado da checagem.
    pub fn is_pixel_onscreen(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// Verifica se um retângulo está visível na tela.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X.
    /// `y` - Posição Y.
    /// `width` - Largura do retângulo.
    /// `height` - Altura do retângulo.
    ///
    /// # Retorna:
    ///
    /// Resultado da checagem.
    pub fn is_onscreen(&self, x: i32, y: i32, width: i32, height: i32) -> bool {
        let left: i32 = x;
        let top: i32 = y;
        let right: i32 = x + width;
        let bottom: i32 = y + height;

        (left >= 0 && left < self.width && top >= 0 && top < self.height)
            || (right > 0 && right < self.width && bottom > 0 && bottom < self.height)
    }

    /// Desenha um único pixel na coordenada especificda.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X do pixel.
    /// `y` - Posição Y do pixel.
    /// `color` - Índice de cor.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn set_pixel(&mut self, mut x: i32, mut y: i32, color: u8) -> bool {
        // A operação não pode seguir adiante se não existirem dados de imagem:
        if self.data.is_empty() {
            return false;
        }

        // Repetir as coordenadas horizontais quando a opçào de wrapping
        // estiver ativada:
        if self.x_wrapping {
            // Calcular wrapping para a posição X:
            if x < 0 {
                x += self.width;
            } else {
                x = (x % self.width).abs();
            }
        }

        // Repetir as coordenadas verticais quando a opçào de wrapping
        // estiver ativada:
        if self.y_wrapping {
            if y < 0 {
                y += self.height;
            } else {
                y = (y % self.height).abs();
            }
        }

        // Desenhar o pixel (apenas se estiver visível na tela):
        if self.is_pixel_onscreen(x, y) {
            self.data[((y * self.width) + x) as usize] = color;
            return true;
        }

        false
    }

    /// Obtém a cor de um único pixel na coordenada especificada.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X do pixel.
    /// `y` - Posição Y do pixel.
    ///
    /// # Retorna:
    ///
    /// O índice de cor do pixel localizado. Caso as coordenadas ultrapassem os
    /// limites da imagem, o índice de transparência é retornado no lugar.
    pub fn get_pixel(&self, mut x: i32, mut y: i32) -> u8 {
        // A operação não pode seguir adiante se não existirem dados de imagem:
        if self.data.is_empty() {
            return self.alpha;
        }

        // Repetir as coordenadas horizontais quando a opçào de wrapping
        // estiver ativada:
        if self.x_wrapping {
            // Calcular wrapping para a posição X:
            if x < 0 {
                x += self.width;
            } else {
                x = (x % self.width).abs();
            }
        }

        // Repetir as coordenadas verticais quando a opçào de wrapping
        // estiver ativada:
        if self.y_wrapping {
            if y < 0 {
                y += self.height;
            } else {
                y = (y % self.height).abs();
            }
        }

        // Retornar o pixel (apenas se estiver visível na tela):
        if self.is_pixel_onscreen(x, y) {
            return self.data[((y * self.width) + x) as usize];
        }

        self.alpha
    }

    /// Recorta um pedaço da imagem e retorna uma cópia.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X de corte.
    /// `y` - Posição Y de corte.
    /// `width` - Largura do pedaço.
    /// `height` - Altura do pedaço.
    ///
    /// # Retorna:
    ///
    /// Um novo objeto de imagem com o pedaço recortado.
    pub fn crop(&self, x: i32, y: i32, mut width: i32, mut height: i32) -> Bitmap {
        // Usar valores positivos...
        width = width.abs();
        height = height.abs();

        // Paleta e pixels da imagem a ser recortada.
        let mut data: Vec<u8> = Vec::new();

        // Copiar pixels da imagem...
        for i in 0..height {
            for j in 0..width {
                data.push(self.get_pixel(x + j, y + i));
            }
        }

        // Objeto de imagem com o pedaço recortado.
        let mut image = Bitmap::new(width, height, self.palette.clone());
        image.metadata = self.metadata;
        image.x_wrapping = self.x_wrapping;
        image.y_wrapping = self.y_wrapping;
        image.alpha = self.alpha;
        image.data = data;

        image
    }

    /// @todo Comentar...
    /// Desenha uma linha.
    ///
    /// Link do código original (em C#):
    /// https://stackoverflow.com/questions/11678693/all-cases-covered-bresenhams-line-algorithm/11683720#11683720
    ///
    /// # Parâmetro(s):
    ///
    /// `x1` - Posição X (1/2).
    /// `y1` - Posição Y (1/2).
    /// `x2` - Posição X (2/2).
    /// `y2` - Posição Y (2/2).
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn line(&mut self, mut x1: i32, mut y1: i32, x2: i32, y2: i32, color: u8) -> bool {
        let w: i32 = x2 - x1;
        let h: i32 = y2 - y1;

        let mut dx1: i32 = 0;
        let mut dy1: i32 = 0;
        let mut dx2: i32 = 0;
        let mut dy2: i32 = 0;

        if w < 0 {
            dx1 = -1;
        } else if w > 0 {
            dx1 = 1;
        }

        if h < 0 {
            dy1 = -1;
        } else if h > 0 {
            dy1 = 1;
        }

        if w < 0 {
            dx2 = -1;
        } else if w > 0 {
            dx2 = 1;
        }

        let mut longest: i32 = w.abs();
        let mut shortest: i32 = h.abs();

        if longest <= shortest {
            longest = h.abs();
            shortest = w.abs();

            if h < 0 {
                dy2 = -1;
            } else if h > 0 {
                dy2 = 1;
            }

            dx2 = 0;
        }

        let mut numerator: i32 = longest >> 1;

        for _i in 0..=longest {
            self.set_pixel(x1, y1, color);

            numerator += shortest;

            if numerator >= longest {
                numerator -= longest;
                x1 += dx1;
                y1 += dy1;
            } else {
                x1 += dx2;
                y1 += dy2;
            }
        }

        true
    }

    /// Desenha um retângulo apenas com bordas.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X inicial.
    /// `y` - Posição Y inicial.
    /// `width` - Largura do retângulo.
    /// `height` - Altura do retângulo.
    /// `color` - Índice de cor.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn rectb(&mut self, x: i32, y: i32, width: i32, height: i32, color: u8) -> bool {
        // Desenhar as 4 bordas do retângulo...
        self.line(x, y, x + width, y, color);
        self.line(x + width, y, x + width, y + height, color);
        self.line(x + width, y + height, x, y + height, color);
        self.line(x, y + height, x, y, color);

        true
    }

    /// Desenha um retângulo preenchido.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X inicial.
    /// `y` - Posição Y inicial.
    /// `width` - Largura do retângulo.
    /// `height` - Altura do retângulo.
    /// `color` - Índice de cor.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u8) -> bool {
        // Preencher retângulo...
        for i in 0..height as i32 {
            for j in 0..width as i32 {
                self.set_pixel(x + j, y + i, color);
            }
        }

        true
    }

    /// Desenha uma imagem. Este método é bem mais complexo que os demais por
    /// incluir diversos atributos auxiliares, como posição de corte e escala.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X inicial.
    /// `y` - Posição Y inicial.
    /// `image` - Imagem a ser desenhada.
    /// `mask` - Caso possua um valor, a imagem é desenhado como uma "sombra" desta cor.
    /// `blit` - Caso possua um valor, a imagem é desenhado apenas dentro desta cor.
    /// `x_scale` - Escala horizontal da imagem.
    /// `y_scale` - Escala vertical da imagem.
    /// `x_start` - Posição X inicial de corte da imagem a ser desenhada.
    /// `y_start` - Posiçao Y inicial de corte da imagem a ser desenhada.
    /// `width` - Largura da imagem a ser desenhada.
    /// `height` - Altura da imagem a ser desenhada.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn draw_image_part(
        &mut self,
        x: i32,
        y: i32,
        image: &Bitmap,
        mask: Option<u8>,
        blit: Option<u8>,
        mut x_scale: i32,
        mut y_scale: i32,
        x_start: i32,
        y_start: i32,
        width: i32,
        height: i32,
    ) -> bool {
        // Índice de cor de transparência do sprite.
        let alpha: u8 = image.get_alpha();

        // As dimensões de escala devem possuir apenas valores positivos:
        x_scale = x_scale.abs();
        y_scale = y_scale.abs();

        // A escala horizontal não pode ser igual a 0:
        if x_scale == 0 {
            x_scale = 1;
        }

        // A escala vertical não pode ser igual a 0:
        if y_scale == 0 {
            y_scale = 1;
        }

        // Percorrer pixels do sprite...
        for i in 0..height * y_scale as i32 {
            for j in 0..width * x_scale as i32 {
                // Pixel do sprite:
                let mut pixel: u8 = image.get_pixel(
                    (j + (x_start * x_scale)) / x_scale,
                    (i + (y_start * y_scale)) / y_scale,
                );

                // Desenhar pixels, exceto seu valor de transparência:
                if pixel != alpha {
                    // Se o sprite for desenhado como uma máscara, o valor do
                    // pixel é substituído, e o sprite será essencialmente
                    // desenhado como uma "sombra":
                    if let Some(mask) = mask {
                        pixel = mask;
                    }

                    // Se o sprite levar em consideração o método de "blitter",
                    // ele só será desenhado na cor apropriada. Assim, o sprite
                    // pode ter elementos na frente e atrás dele:
                    if let Some(blit) = blit {
                        if self.get_pixel(x + j + x_start, y + i + y_start) == blit {
                            self.set_pixel(x + j, y + i, pixel);
                        }
                    }
                    // Sem o "blitter", o sprite é imediatamente desenhado por
                    // cima da imagem:
                    else {
                        self.set_pixel(x + j, y + i, pixel);
                    }
                }
            }
        }

        true
    }

    /// @todo Comentar isso aqui e possivelmente fazer versões AINDA MAIS SIMPLIFICADAS!
    /// Desenha uma imagem completa. É o método mais simples e direto.
    ///
    /// # Parâmetro(s):
    ///
    /// `x` - Posição X inicial.
    /// `y` - Posição Y inicial.
    /// `image` - Imagem a ser desenhada.
    /// `mask` - Caso possua um valor, a imagem é desenhado como uma "sombra" desta cor.
    /// `blit` - Caso possua um valor, a imagem é desenhado apenas dentro desta cor.
    /// `x_scale` - Escala horizontal da imagem.
    /// `y_scale` - Escala vertical da imagem.
    ///
    /// # Retorna:
    ///
    /// Um valor booleano que indica êxito (`true`) ou falha (`false`).
    pub fn draw_image(
        &mut self,
        x: i32,
        y: i32,
        image: &Bitmap,
        mask: Option<u8>,
        blit: Option<u8>,
        x_scale: i32,
        y_scale: i32,
    ) -> bool {
        self.draw_image_part(
            x,
            y,
            image,
            mask,
            blit,
            x_scale,
            y_scale,
            0,
            0,
            image.get_width(),
            image.get_height(),
        )
    }

    /// @todo Comentar e testar depois.
    pub fn draw_frame(
        &mut self,
        x: i32,
        y: i32,
        image: &Bitmap,
        mask: Option<u8>,
        blit: Option<u8>,
        x_scale: i32,
        y_scale: i32,
        frame: &Frame,
    ) -> bool {
        let x_start = frame.x as i32;
        let y_start = frame.y as i32;
        let width = frame.w as i32;
        let height = frame.h as i32;

        self.draw_image_part(
            x, y, image, mask, blit, x_scale, y_scale, x_start, y_start, width, height,
        )
    }

    /// `@getter`
    /// Largura desta imagem.
    pub fn get_width(&self) -> i32 {
        self.width
    }

    /// `@getter`
    /// Altura desta imagem.
    pub fn get_height(&self) -> i32 {
        self.height
    }

    /// `@getter`
    /// Metadados desta imagem.
    pub fn get_metadata(&self) -> u32 {
        self.metadata
    }

    /// `@setter`
    ///
    /// # Parâmetro(s):
    ///
    /// `metadata` - Índice de cor usado para o canal de transparência desta imagem.
    pub fn set_metadata(&mut self, metadata: u32) {
        self.metadata = metadata;
    }

    /// `@getter`
    /// Paleta de cores desta imagem (até 256 cores).
    pub fn get_palette(&self) -> &Palette {
        &self.palette
    }

    /// `@setter`
    ///
    /// # Parâmetro(s):
    ///
    /// `palette` - Paleta de cores desta imagem (até 256 cores).
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// `@getter`
    /// Índice de cor usado para o canal de transparência desta imagem.
    pub fn get_alpha(&self) -> u8 {
        self.alpha
    }

    /// `@setter`
    ///
    /// # Parâmetro(s):
    ///
    /// `alpha` - Índice de cor usado para o canal de transparência desta imagem.
    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    /// Define os estados de wrapping horizontal e vertical.
    ///
    /// # Parâmetro(s):
    ///
    /// `x_wrapping` - Estado do wrapping horizontal.
    /// `y_wrapping` - Estado do wrapping vertical.
    pub fn set_wrapping(&mut self, x_wrapping: bool, y_wrapping: bool) {
        self.x_wrapping = x_wrapping;
        self.y_wrapping = y_wrapping;
    }

    /// `@getter`
    /// Estado do wrapping horizontal.
    pub fn is_x_wrapping(&self) -> bool {
        self.x_wrapping
    }

    /// `@setter`
    ///
    /// # Parâmetro(s):
    ///
    /// `x_wrapping` - Estado do wrapping horizontal.
    pub fn set_x_wrapping(&mut self, x_wrapping: bool) {
        self.x_wrapping = x_wrapping;
    }

    /// `@getter`
    /// Estado do wrapping vertical.
    pub fn is_y_wrapping(&self) -> bool {
        self.y_wrapping
    }

    /// `@setter`
    ///
    /// # Parâmetro(s):
    ///
    /// `y_wrapping` - Estado do wrapping vertical.
    pub fn set_y_wrapping(&mut self, y_wrapping: bool) {
        self.y_wrapping = y_wrapping;
    }
}

impl GetRgba for Bitmap {
    fn get_rgba(&self, x: usize, y: usize) -> u32 {
        let px = self.get_pixel(x as i32, y as i32);
        let c = self.palette.get_color(px as usize);
        // FIXME: figure out why palette gives funny RGB values
        let mut g = Colour::green();
        g.set_alpha(0);
        c.to_rgba().saturating_sub(g.to_rgba())
    }
}
