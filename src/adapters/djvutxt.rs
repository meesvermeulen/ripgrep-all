use super::*;
use lazy_static::lazy_static;
use spawning::SpawningFileAdapter;
use std::io::BufReader;
use std::process::Command;

static EXTENSIONS: &[&str] = &["djvu"];

lazy_static! {
    static ref METADATA: AdapterMeta = AdapterMeta {
        name: "djvutext".to_owned(),
        version: 1,
        description: "Uses djvutext (from DjVuLibre) to extract plain text from DjVu files"
            .to_owned(),
        recurses: false,
        fast_matchers: EXTENSIONS
            .iter()
            .map(|s| FastMatcher::FileExtension(s.to_string()))
            .collect(),
        slow_matchers: Some(vec![SlowMatcher::MimeType("application/djvu".to_owned())])
    };
}

#[derive(Default)]
pub struct DjvutxtAdapter;

impl DjvutxtAdapter {
    pub fn new() -> DjvutxtAdapter {
        DjvutxtAdapter
    }
}

impl GetMetadata for DjvutxtAdapter {
    fn metadata(&self) -> &AdapterMeta {
        &METADATA
    }
}

impl SpawningFileAdapter for DjvutxtAdapter {
    fn get_exe(&self) -> &str {
        "djvutxt"
    }

    fn command(&self, _filepath_hint: &Path, mut cmd: Command) -> Command {
        cmd.arg("-");
        cmd
    }

    fn postproc(line_prefix: &str, inp: &mut dyn Read, oup: &mut dyn Write) -> Fallible<()> {
        // String returned from djvutxt may return invalid utf8, sanitize it first.
        let mut vec = Vec::<u8>::new();
        inp.read_to_end(&mut vec)?;
        let sanitized_string = String::from_utf8_lossy(&vec).into_owned();

        // prepend Page X to each line
        let mut page = 1;
        for line in BufReader::new(sanitized_string.as_bytes()).lines() {
            let mut line = line?;
            if line.contains('\x0c') {
                // page break
                line = line.replace('\x0c', "");
                page += 1;
                if line.is_empty() {
                    continue;
                }
            }
            oup.write_all(format!("{}Page {}: {}\n", line_prefix, page, line).as_bytes())?;
        }
        Ok(())
    }
}
