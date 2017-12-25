# pit-rs

[pit](https://github.com/cho45/pit) for [Rust](https://www.rust-lang.org/)

## Usage

```
extern crate pit;

use pit::Pit;

fn main() {
    let p = Pit::new();
    let config = p.get("twitter.com");
    match config {
        None => {
            println!("not provide config value");
            return;
        },
        Some(_) => {},
    }

    let config = config.unwrap();
    let username = config.get("username").unwrap();
    let password = config.get("password").unwrap();
    println!("username={}, password={}", username, password);
}
```


# for developer
```
$ cargo test -- --test-threads=1
```
