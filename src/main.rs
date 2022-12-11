use std::io::Write;
use std::process::Command;

use clap::Parser;
use url::Url;

#[derive(Parser)]
enum Cli {
    /// Print the type of a Git object.
    Type {
        uri: String,
    },
    Show {
        uri: String,
    },
}

#[derive(Debug)]
enum GitUri<'a> {
    Object { hash: &'a str },
    Blob { hash: &'a str },
    Commit { hash: &'a str },
    Tree { hash: &'a str },
}

fn check_object_type(hash: &str, expected_type: &str) {
    let output = Command::new("git")
        .args(["cat-file", "-t", hash])
        .output()
        .unwrap();

    if output.status.success() {
        let actual_type: &str = std::str::from_utf8(&output.stdout).unwrap().trim();
        if expected_type != actual_type {
            panic!("{hash} is not a {expected_type} (it's a {})", actual_type);
        }
    } else {
        eprint!("git error: ");
        std::io::stderr()
            .write_all(output.stderr.as_slice())
            .unwrap();
        std::process::exit(1);
    }
}

fn decode_url(url: &Url) -> GitUri {
    if url.scheme() == "git" {
        let path_parts = url.path().split(':').collect::<Vec<_>>();
        match path_parts.get(0) {
            None => {
                panic!("invalid git URI: {:?}", url);
            }
            Some(path_type) => match *path_type {
                "object" => match path_parts.get(1) {
                    None => panic!("invalid git object URI: {:?}", url),
                    Some(hash) => {
                        if path_parts.len() > 2 {
                            panic!("invalid git object URI: {:?}", url);
                        }
                        GitUri::Object { hash: *hash }
                    }
                },
                "blob" => match path_parts.get(1) {
                    None => panic!("invalid git blob URI: {:?}", url),
                    Some(hash) => {
                        if path_parts.len() > 2 {
                            panic!("invalid git blob URI: {:?}", url);
                        }
                        GitUri::Blob { hash: *hash }
                    }
                },
                "tree" => match path_parts.get(1) {
                    None => panic!("invalid git tree URI: {:?}", url),
                    Some(hash) => {
                        if path_parts.len() > 2 {
                            panic!("invalid git tree URI: {:?}", url);
                        }
                        GitUri::Tree { hash: *hash }
                    }
                },
                "commit" => match path_parts.get(1) {
                    None => panic!("invalid git commit URI: {:?}", url),
                    Some(hash) => {
                        if path_parts.len() > 2 {
                            panic!("invalid git commit URI: {:?}", url);
                        }
                        GitUri::Commit { hash: *hash }
                    }
                },
                _ => panic!(
                    "invalid type \"{}\", expected one of: blob, commit, object, tree",
                    path_type
                ),
            },
        }
    } else {
        panic!("expected \"git\" scheme, got \"{}\"", url.scheme());
    }
}

fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::Type { uri } => {
            let url = Url::parse(&uri).unwrap();
            let git_uri = decode_url(&url);
            match git_uri {
                GitUri::Object { hash } => {
                    let output = Command::new("git")
                        .args(["cat-file", "-t", hash])
                        .output()
                        .unwrap();

                    if output.status.success() {
                        std::io::stdout()
                            .write_all(output.stdout.as_slice())
                            .unwrap();
                    } else {
                        eprint!("git error: ");
                        std::io::stderr()
                            .write_all(output.stderr.as_slice())
                            .unwrap();
                        std::process::exit(1);
                    }
                }
                GitUri::Blob { hash } => {
                    check_object_type(hash, "blob");
                    println!("blob");
                }
                GitUri::Commit { hash } => {
                    check_object_type(hash, "commit");
                    println!("commit");
                }
                GitUri::Tree { hash } => {
                    check_object_type(hash, "tree");
                    println!("tree");
                }
            }
        }
        Cli::Show { uri } => {
            let url = Url::parse(&uri).unwrap();
            let git_uri = decode_url(&url);
            let hash = match git_uri {
                GitUri::Object { hash } => hash,
                GitUri::Blob { hash } => {
                    check_object_type(hash, "blob");
                    hash
                }
                GitUri::Commit { hash } => {
                    check_object_type(hash, "commit");
                    hash
                }
                GitUri::Tree { hash } => {
                    check_object_type(hash, "tree");
                    hash
                }
            };

            let output = Command::new("git")
                .args(["cat-file", "-p", hash])
                .output()
                .unwrap();

            if output.status.success() {
                std::io::stdout()
                    .write_all(output.stdout.as_slice())
                    .unwrap();
            } else {
                eprint!("git error: ");
                std::io::stderr()
                    .write_all(output.stderr.as_slice())
                    .unwrap();
                std::process::exit(1);
            }
        }
    }
}
