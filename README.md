# Envar

Envar is a simple library to manage environment variables in Rust. 
It provides a derive macro to automatically parse environment variables into a struct at compile time.
The advantage of this approach is that it allows you to catch errors at compile time, rather than getting a runtime error when trying to access an environment variable that doesn't exist.

## Usage
```rust

 use envar::Envar;
 #[derive(Envar)]
 struct Config {
    #[env = "DB_CONNECTION_PORT"]
    port: u16,
    #[env = "DB_CONNECTION_HOST"]
    host: String,
    debug: Option<bool>,
}
 fn main() {
   let config = Config::new();
   println!("Port: {}", config.port);
   println!("Host: {}", config.host);
   // If PORT and HOST are not found in the environment, the program will not compile.
   // If DEBUG is not found, it will be None.
   println!("Debug: {:?}", config.debug);
 }
```

