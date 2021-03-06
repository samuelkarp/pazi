use harness::autojumpers::Autojumper;
use std::fs;
use std::io::Write;
use std::path::Path;

pub enum Shell {
    Bash,
    Zsh,
    #[allow(dead_code)]
    Conch,
}

pub struct ShellCmd<'a> {
    pub cmd: &'a str,
    pub env: Vec<(&'a str, String)>,
}

impl Shell {
    pub fn name(&self) -> &'static str {
        match self {
            &Shell::Bash => "bash",
            &Shell::Zsh => "zsh",
            &Shell::Conch => unimplemented!(),
        }
    }

    pub fn from_str(name: &str) -> Self {
        match name {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            _ => unimplemented!(),
        }
    }

    pub fn setup(&self, root: &Path, autojump: &Autojumper, ps1: &str, preinit: &str) {
        let rc_file = match *self {
            Shell::Bash | Shell::Zsh => root.join(format!("home/pazi/.{}rc", self.name())),
            Shell::Conch => unimplemented!(),
        };

        let rc_template = match *self {
            Shell::Bash => format!(
                r#"#Autogenerated by pazi integ tests
set -e
{preinit}
export PS1="{ps1}" # sep so we know when our commands finished
export PATH=$PATH:$(dirname "{bin_path}")
{init}
"#,
                bin_path=autojump.bin_path(),
                init=autojump.init_for(self),
                ps1=ps1,
                preinit=preinit,
            ),
            Shell::Zsh => format!(
                r#"#Autogenerated by pazi integ tests
set -e
unsetopt zle
{preinit}
export PS1="{ps1}" # sep so we know when our commands finished
export PATH=$PATH:$(dirname "{bin_path}")
{init}
"#,
                bin_path=autojump.bin_path(),
                init=autojump.init_for(self),
                ps1=ps1,
                preinit=preinit,
            ),
            Shell::Conch => unimplemented!(),
        };

        fs::create_dir_all(root.join("home/pazi")).unwrap();
        fs::File::create(rc_file)
            .unwrap()
            .write_all(rc_template.as_bytes())
            .unwrap();
    }

    pub fn command(&self, root: &Path) -> ShellCmd {
        let home = root.join("home/pazi").to_string_lossy().to_string();
        ShellCmd {
            cmd: self.name(),
            env: vec![ ("HOME", home), ],
        }
    }
}
