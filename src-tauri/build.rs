use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&*shellexpand::tilde("~/.pandoc/templates"))?;

    let pandoc_template_file_path = shellexpand::tilde("~/.pandoc/templates/malp_default_md_html.html");

    let mut pandoc_template_file = std::fs::File::create(&*pandoc_template_file_path)?;

    pandoc_template_file.write_all(b"\
<header>
    <h1> $title$ </h1>
    <h2> $subtitle$ </h2>
    <p> $author$ <p>
</header>

$body$")?;
    tauri_build::build();
    Ok(())
}
