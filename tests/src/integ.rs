extern crate pazi;
extern crate tempdir;

use harness::{Fasd, HarnessBuilder, Pazi, Shell};
use integ::pazi::shells::SUPPORTED_SHELLS;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use tempdir::TempDir;

#[test]
fn it_jumps() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_jumps_shell(&s);
    }
}

fn it_jumps_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();

    h.create_dir(&slash_tmp);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("tmp"), slash_tmp);
}

#[test]
fn it_jumps_to_exact_directory() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_jumps_to_exact_directory_shell(&s);
    }
}

fn it_jumps_to_exact_directory_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();
    let unvisited_dir_path = slash_tmp_path.join("asdf");
    let unvisited_dir = unvisited_dir_path.to_string_lossy();

    h.create_dir(&unvisited_dir);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("asdf"), unvisited_dir);
}

#[test]
fn it_jumps_to_more_frecent_items() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_jumps_to_more_frecent_items_shell(&s);
    }
}

fn it_jumps_to_more_frecent_items_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();
    let a_dir_path = root.join("a/tmp");
    let b_dir_path = root.join("b/tmp");
    let a_dir = a_dir_path.to_string_lossy();
    let b_dir = b_dir_path.to_string_lossy();

    h.create_dir(&a_dir);
    h.create_dir(&b_dir);
    // Visiting 'b' more recently means it shouldbe more frecent.
    h.visit_dir(&a_dir);
    sleep(Duration::from_millis(5));
    h.visit_dir(&b_dir);
    assert_eq!(h.jump("tmp"), b_dir);

    // Visiting 'a' more often should make it more 'frecent'
    for _ in 0..10 {
        h.visit_dir(&a_dir);
        // visit another directory between since 'cd' to the same directory you're in doesn't
        // necessarily increase frecency.
        h.visit_dir(&root.to_string_lossy());
    }
    h.visit_dir(&b_dir);
    assert_eq!(h.jump("tmp"), a_dir);
}

#[test]
fn it_imports_from_fasd() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_imports_from_fasd_shell(&s);
    }
}

fn it_imports_from_fasd_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();

    {
        let mut fasd = HarnessBuilder::new(&root, &Fasd, shell).finish();
        fasd.create_dir(&root.join("tmp").to_string_lossy());
        // visit twice because fasd uses 'history 1' to do stuff in bash... which means yeah, it's
        // 1-command-delayed
        fasd.visit_dir(&root.join("tmp").to_string_lossy());
        fasd.visit_dir(&root.join("tmp").to_string_lossy());
    }

    {
        let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();
        assert_eq!(
            h.run_cmd("pazi import fasd").trim(),
            "imported 1 items from fasd (out of 1 in its db)"
        );
        assert_eq!(h.jump("tmp"), root.join("tmp").to_string_lossy());
    }
}

#[test]
fn it_ignores_dead_dirs_on_cd() {
    for shell in SUPPORTED_SHELLS.iter() {
        println!("testing: {}", shell);
        let s = Shell::from_str(shell);
        it_ignores_dead_dirs_on_cd_shell(&s);
    }
}

fn it_ignores_dead_dirs_on_cd_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();

    h.create_dir(&root.join("1/tmp").to_string_lossy());
    h.create_dir(&root.join("2/tmp").to_string_lossy());

    h.visit_dir(&root.join("1/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());

    assert_eq!(h.jump("tmp"), root.join("2/tmp").to_string_lossy());
    h.delete_dir(&root.join("2/tmp").to_string_lossy());
    assert_eq!(h.jump("tmp"), root.join("1/tmp").to_string_lossy());
}

#[test]
fn it_prints_list_on_lonely_z() {
    // running just 'z' or just 'pazi' should print a directory listing, not error
    for shell in SUPPORTED_SHELLS.iter() {
        let s = Shell::from_str(shell);
        it_prints_list_on_lonely_z_shell(&s);
    }
}

fn it_prints_list_on_lonely_z_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();

    h.create_dir(&root.join("1/tmp").to_string_lossy());
    h.create_dir(&root.join("2/tmp").to_string_lossy());
    h.visit_dir(&root.join("1/tmp").to_string_lossy());
    h.visit_dir(&root.join("2/tmp").to_string_lossy());

    let z_res = h.run_cmd("z");
    let pazi_res = h.run_cmd("pazi view");

    assert_eq!(z_res, pazi_res);
    assert!(z_res.contains(&root.join("1/tmp").to_string_lossy().to_string()));
}

// Regression test for https://github.com/euank/pazi/issues/41
#[test]
fn it_handles_existing_bash_prompt_command() {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let prompt_cmd = r#"
MY_PROMPT=1
PROMPT_COMMAND='printf -v MY_PROMPT_OUT "\033k%s\033\\" "${MY_PROMPT}"'
"#;
    let mut h = HarnessBuilder::new(&root, &Pazi, &Shell::Bash)
        .preinit(&prompt_cmd)
        .finish();
    let slash_tmp_path = root.join("tmp");
    let slash_tmp = slash_tmp_path.to_string_lossy();

    h.create_dir(&slash_tmp);
    h.visit_dir(&slash_tmp);
    assert_eq!(h.jump("tmp"), slash_tmp);

    h.run_cmd("MY_PROMPT=2");
    let check_prompt_out_cmd = r#"printf "%q\n" "${MY_PROMPT_OUT}""#;
    let expected_prompt_out = r#"$'\Ek2\E\\'"#;
    assert_eq!(h.run_cmd(check_prompt_out_cmd), expected_prompt_out);
}

// Test for https://github.com/euank/pazi/issues/49
#[test]
fn it_handles_help_output() {
    for shell in SUPPORTED_SHELLS.iter() {
        let s = Shell::from_str(shell);
        it_handles_help_output_shell(&s);
    }
}

fn it_handles_help_output_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();
    let help1 = h.run_cmd("pazi --help && echo $?");
    let help2 = h.run_cmd("z -h && echo $?");
    let help3 = h.run_cmd("z --help && echo $?");
    assert!(help1.contains("USAGE:"), help1);
    assert!(help2.contains("USAGE:"), help2);
    assert!(help1.ends_with("\n0"));
    assert!(help2.ends_with("\n0"));

    assert_eq!(help2, help3);
}

// Test for https://github.com/euank/pazi/issues/60
// and https://github.com/euank/pazi/issues/70
#[test]
fn it_handles_things_that_look_like_subcommands() {
    for shell in SUPPORTED_SHELLS.iter() {
        let s = Shell::from_str(shell);
        it_handles_things_that_look_like_subcommands_shell(&s);
    }
}

fn it_handles_things_that_look_like_subcommands_shell(shell: &Shell) {
    let tmpdir = TempDir::new("pazi_integ").unwrap();
    let root = tmpdir.path();
    let mut h = HarnessBuilder::new(&root, &Pazi, shell).finish();

    // map of <DirectoryName, JumpTarget>
    // Each will be tested that given a frecent directory of that name, a jump of the given target
    // will end up there correctly.
    let map: HashMap<_, _> = vec![
        ("ignition", "igni"),
        ("igni", "igni"),
        ("initialize", "init"),
        ("--help", "help"),
        ("import", "import"),
    ].into_iter()
    .collect();

    for (dir, jump) in map {
        let dirname = root.join(dir).into_os_string().into_string().unwrap();
        h.create_dir(&dirname);
        h.visit_dir(&dirname);
        h.visit_dir(&root.to_string_lossy());
        assert_eq!(h.jump(jump), dirname);
        h.delete_dir(&dirname);
    }
}
