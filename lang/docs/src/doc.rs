use std::fs;
use std::io;
use std::path::PathBuf;

const HTML_END: &str = "</body></html>";

fn html_start() -> String {
    "<!DOCTYPE html>
    <html>
    <head>
        <meta charset=\"UTF-8\">
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
        <title>Document</title>
    </head>
    <body>".to_string()
}

fn generate_html(cmd: Args) -> String {
    let html = format!("
<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>Code Display</title>
    <!-- Verlinke die externe CSS-Datei -->
    <link rel=\"stylesheet\" href=\"style.css\">
</head>
<body>
    <div>
        <h1>{}</h1>
        <pre><code>
<span class=\"keyword\">data</span> Nat {{ Z, S(n: Nat) }}

<span class=\"keyword\">data</span> NotZero(n: Nat) {{
    SNotZero(n: Nat): NotZero(S(n))
}}

<span class=\"keyword\">def</span> NotZero(Z).elim_zero(a: Type): a {{ SNotZero(n) absurd }}

<span class=\"keyword\">data</span> Bot {{ }}

<span class=\"keyword\">data</span> Foo(a: Type) {{
    Ok(a: Type, x: a): Foo(a),
    Absurd(x: NotZero(Z)): Foo(Bot)
}}

<span class=\"keyword\">def</span> Foo(a).elim(a: Type): a {{
    Ok(a, x) =&gt; x,
    Absurd(x) =&gt; x.elim_zero(Bot)
}}
        </code></pre>
    </div>
</body>
</html>
    ", cmd.filepath.to_string_lossy());
    html.to_string()
}
   

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, default_value_t = 80)]
    width: usize,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn compute_output_stream(cmd: &Args) -> Box<dyn io::Write> {
    match &cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => {
            let path = format!("target_pol/docs/{}", cmd.filepath.file_name().unwrap().to_string_lossy());
            let mut fp = PathBuf::from(path);
            fp.set_extension("html");
            Box::new(fs::File::create(fp).expect("Failed to create file"))
        }
    }
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut stream: Box<dyn io::Write> = compute_output_stream(&cmd);

    stream.write_all(html_start().as_bytes()).unwrap();
    stream.write_all(generate_html(cmd).as_bytes()).unwrap();
    stream.write_all(HTML_END.as_bytes()).unwrap();
    Ok(())
}