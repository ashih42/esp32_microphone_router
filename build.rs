fn main() {
    // 1. Tell cargo to rerun this script if the .env file changes.
    println!("cargo:rerun-if-changed=.env");

    // 2. Load the .env file.
    match dotenvy::dotenv() {
        Ok(path) => {
            println!(
                "cargo:warning=.env loaded successfully from: {}",
                path.display()
            );

            // 3. Loop through every variable inside the .env file
            // and explicitly forward it to the main rustc compiler context.
            for item in dotenvy::dotenv_iter().unwrap() {
                let (key, value) = item.unwrap();
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
        Err(err) => {
            panic!("CRITICAL: Could not load .env file: {}", err);
        }
    };

    embuild::espidf::sysenv::output();
}
