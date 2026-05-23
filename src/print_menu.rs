use std::io::{self, Write};

pub fn print_menu() {
    println!("\n--- Notebook Menu \n 
    1. Create New Note \n
    2. View All Notes \n 
    4. Delete Note \n
    5. Exit \n
    0. Menu");
    /*    
    3. Update Note \n */
    io::stdout().flush().unwrap(); // WATCH!
}


