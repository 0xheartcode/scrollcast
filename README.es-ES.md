# Scrollcast

Una herramienta de CLI y librería en Rust para convertir repositorios de Git en documentos formateados (PDF, EPUB, HTML, Markdown) con resaltado de sintaxis.

## Características

- Convertir repositorios a múltiples formatos de salida
- Resaltado de sintaxis utilizando Syntect
- Integración con Git y soporte para `.gitignore`
- Procesamiento eficiente de memoria mediante fragmentación (chunking)
- Implementación pura en Rust

## Instalación

### Desde el código fuente

```bash
git clone https://github.com/0xheartcode/scrollcast
cd scrollcast
cargo install --path .
```

### Como librería

Añade a tu `Cargo.toml`:

```toml
[dependencies]
scrollcast = "0.1.0"
```

## Uso

### Línea de comandos

```bash
# Convertir a PDF
scrollcast /path/to/repo -o output.pdf

# Convertir a HTML con un tema diferente
scrollcast /path/to/repo -o output.html -f html -t zenburn

# Procesar todos los archivos (ignorar .gitignore)
scrollcast /path/to/repo -o output.pdf --no-gitignore

# Excluir directorios específicos
scrollcast /path/to/repo -o output.pdf --ignore target --ignore node_modules
```

### Librería

```rust
use scrollcast::{FileProcessor, MarkdownGenerator, OutputFormat, create_renderer, DocumentMetadata};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let input_path = Path::new("./my-repo");
    let output_path = Path::new("./output.pdf");
    
    let mut processor = FileProcessor::new(input_path, true, Vec::new())?;
    let files = processor.discover_files().await?;
    
    let generator = MarkdownGenerator::new();
    let markdown = generator.generate_markdown(&files, input_path, false).await?;
    
    let metadata = DocumentMetadata {
        title: "Repository Export".to_string(),
        author: "Scrollcast".to_string(),
        created_at: chrono::Utc::now(),
    };
    
    let mut renderer = create_renderer(OutputFormat::Pdf, "kate".to_string())?;
    renderer.render(&markdown, output_path, &metadata).await?;
    
    Ok(())
}
```

## Opciones de la Línea de Comandos

```
Usage: scrollcast [OPTIONS] [input]

Arguments:
  [input]  Directorio de entrada (repositorio git o carpeta regular)

Options:
  -o, --output <output>                Ruta del archivo de salida
  -f, --format <format>                Formato de salida [predeterminado: pdf] [valores posibles: pdf, epub, html, markdown]
  -t, --theme <theme>                  Tema de resaltado de sintaxis [predeterminado: kate]
      --no-gitignore                   Ignorar archivos .gitignore y procesar todos los archivos
      --no-toc                         No incluir tabla de contenidos
      --list-themes                    Listar temas de resaltado de sintaxis disponibles
      --list-languages                 Listar lenguajes de programación soportados
  -y, --yes                            Omitir confirmaciones
      --ignore <DIR>                   Ignorar directorios específicos (puede usarse varias veces)
  -v, --verbose                        Habilitar registro detallado (verbose)
      --chunk-size <chunk-size>        Procesar archivos en fragmentos [predeterminado: 20]
      --memory-limit <memory-limit>    Uso máximo de memoria en MB
      --max-file-size <max-file-size>  Tamaño máximo de archivo a procesar en MB [predeterminado: 50]
  -h, --help                           Mostrar ayuda
  -V, --version                        Mostrar versión
```

## Formatos de Salida

- **PDF**: Gráficos vectoriales utilizando `printpdf`
- **EPUB**: Documentos ajustables utilizando `epub-builder`
- **HTML**: Archivos independientes con CSS embebido
- **Markdown**: Markdown limpio con resaltado de sintaxis

## Resaltado de Sintaxis

Scrollcast utiliza Syntect para el resaltado de sintaxis con soporte para lenguajes de programación comunes, incluyendo Rust, JavaScript, Python, Go, Java, C/C++, y muchos otros.

Temas disponibles:
- kate (predeterminado)
- pygments
- zenburn
- breezedark
- espresso
- monochrome
- haddock
- tango

Usa `--list-themes` y `--list-languages` para ver todas las opciones disponibles.

## Procesamiento de Archivos

### Exclusiones Automáticas

La herramienta excluye automáticamente:
- Directorios de control de versiones (`.git`, `.svn`)
- Salidas de compilación (`target`, `dist`, `build`)
- Dependencias (`node_modules`, `vendor`)
- Archivos de IDE (`.vscode`, `.idea`)
- Archivos binarios y archivos comprimidos

### Integración con Git

- Respeta el archivo `.gitignore` por defecto
- Usa `--no-gitignore` para procesar todos los archivos
- Detecta automáticamente repositorios de Git

## Rendimiento

Para repositorios grandes, Scrollcast ofrece varias opciones:
- `--chunk-size`: Procesar archivos en lotes más pequeños
- `--memory-limit`: Limitar el uso de memoria
- `--max-file-size`: Omitir archivos excesivamente grandes

## Dependencias

La librería utiliza estas dependencias principales:
- `syntect` - Resaltado de sintaxis
- `printpdf` - Generación de PDF
- `epub-builder` - Creación de EPUB
- `pulldown-cmark` - Procesamiento de Markdown
- `git2` - Integración con Git

## Licencia

MIT

## Contribuir

Las contribuciones son bienvenidas. Por favor, asegúrate de que el código siga las convenciones de Rust e incluya las pruebas apropiadas.
🦀 0xheartcode
