use std::collections::HashMap;
use std::io::Write;
use std::process::Command;

use clap::Parser;
use url::Url;

#[derive(Parser)]
enum Cli {
    /// Print the type of a Git object.
    Type { uri: String },
    /// Print the content of a Git object.
    Show { uri: String },
}

#[derive(Debug)]
enum GitUri<'a> {
    Object {
        hash: &'a str,
    },
    Blob {
        hash: &'a str,
    },
    Commit {
        hash: &'a str,
        path: Option<CommitPath<'a>>,
    },
    Tree {
        hash: &'a str,
        path: Vec<&'a str>,
    },
}

#[derive(Debug)]
enum CommitPath<'a> {
    Tree { path: Vec<&'a str> },
    Parent { path: Vec<&'a str> },
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
                    Some(rest) => {
                        if path_parts.len() > 2 {
                            panic!("invalid git tree URI: {:?}", url);
                        }
                        let segments = rest.split('/').collect::<Vec<_>>();
                        GitUri::Tree {
                            hash: segments[0],
                            path: Vec::from(&segments[1..]),
                        }
                    }
                },
                "commit" => match path_parts.get(1) {
                    None => panic!("invalid git commit URI: {:?}", url),
                    Some(hash) => {
                        let path = path_parts.get(2).copied().map(|part| match part {
                            "tree" => CommitPath::Tree {
                                path: Vec::from(&path_parts[3..]),
                            },
                            "parent" => CommitPath::Parent {
                                path: Vec::from(&path_parts[3..]),
                            },
                            _ => panic!("invalid git commit URI: {:?}", url),
                        });

                        GitUri::Commit { hash: *hash, path }
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

fn read_object(hash: &str) -> String {
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()
        .unwrap();

    if output.status.success() {
        String::from(std::str::from_utf8(&output.stdout).unwrap().trim())
    } else {
        eprint!("git error: ");
        std::io::stderr()
            .write_all(output.stderr.as_slice())
            .unwrap();
        std::process::exit(1);
    }
}

fn read_object_type(hash: &str) -> String {
    let output = Command::new("git")
        .args(["cat-file", "-t", hash])
        .output()
        .unwrap();

    if output.status.success() {
        String::from(std::str::from_utf8(&output.stdout).unwrap().trim())
    } else {
        eprint!("git error: ");
        std::io::stderr()
            .write_all(output.stderr.as_slice())
            .unwrap();
        std::process::exit(1);
    }
}

fn read_tree_object_hash(hash: &str, path: &[&str]) -> String {
    fn tree_object_hash(hash: &str, name: &str) -> Option<String> {
        check_object_type(hash, "tree");

        let tree_str = read_object(hash);
        let name_to_hash = tree_str
            .lines()
            .map(|line| {
                let columns = line.split(' ').collect::<Vec<_>>();
                let hash_and_name = columns[2];
                let (hash, name) = hash_and_name.split_once('\t').unwrap();
                (name, hash)
            })
            .collect::<HashMap<_, _>>();

        name_to_hash.get(name).map(|hash| String::from(*hash))
    }

    let mut hash = String::from(hash);
    for i in 0..path.len() {
        let name = path[i];
        match tree_object_hash(&hash, name) {
            Some(new_hash) => {
                hash = new_hash;
            }
            None => panic!(
                "{name} not found in git:tree:{hash}{}",
                path[0..i]
                    .iter()
                    .fold(String::from(""), |mut acc, segment| {
                        acc.push('/');
                        acc.push_str(segment);
                        acc
                    })
            ),
        }
    }
    hash
}

fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::Type { uri } => {
            let url = Url::parse(&uri).unwrap();
            let git_uri = decode_url(&url);
            match git_uri {
                GitUri::Object { hash } => {
                    println!("{}", read_object_type(hash))
                }
                GitUri::Blob { hash } => {
                    check_object_type(hash, "blob");
                    println!("blob");
                }
                GitUri::Commit { hash, path } => {
                    check_object_type(hash, "commit");
                    println!("commit");
                }
                GitUri::Tree { hash, path } => {
                    let hash = read_tree_object_hash(hash, &path);
                    println!("{}", read_object_type(&hash));
                }
            }
        }
        Cli::Show { uri } => {
            let url = Url::parse(&uri).unwrap();
            let git_uri = decode_url(&url);
            match git_uri {
                GitUri::Object { hash } => {
                    println!("{}", read_object(hash))
                }
                GitUri::Blob { hash } => {
                    check_object_type(hash, "blob");
                    println!("{}", read_object(hash))
                }
                GitUri::Commit { hash, path } => {
                    check_object_type(hash, "commit");
                    println!("{}", read_object(hash))
                }
                GitUri::Tree { hash, path } => {
                    check_object_type(hash, "tree");
                    let hash = read_tree_object_hash(hash, &path);
                    println!("{}", read_object(&hash))
                }
            };
        }
    }
}
