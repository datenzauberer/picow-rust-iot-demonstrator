use std::env;
use std::process::Command;

fn main() {
    // Set Directory to Workspace level (this is needed for creating the database)
    let mut dir = env::current_dir().expect("Failed to get current directory");
    dir.pop();
    env::set_current_dir(&dir).expect("Failed to set current directory");

    // Load the .env file
    dotenv::dotenv().ok();

    Command::new("pwd")
        .output()
        .expect("failed to execute process");
    // Call the function with different sets of arguments for each sqlx operation
    execute_sqlx(&["db", "create"]);
    execute_sqlx(&["migrate", "run", "--source", "iot-db-accessor/migrations"]);
}

fn execute_sqlx(args: &[&str]) {
    // Read the DATABASE_URL environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Execute the sqlx command with the provided arguments
    let status = Command::new("sqlx")
        .env("DATABASE_URL", &database_url)
        .args(args)
        .status()
        .unwrap_or_else(|_| panic!("Failed to execute `sqlx` with arguments: {:?}", args));

    if !status.success() {
        eprintln!("`sqlx` command with arguments {:?} failed.", args);
        std::process::exit(1);
    }
}
