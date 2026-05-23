use crate::io_handler::io_handler;
use rusqlite::{Connection, Result, Row};

pub fn create_note(conn: &Connection) -> Result<()> {
    let title = io_handler("Enter note title");
    let content = io_handler("Enter note content");
    //We may need to pass the time variable?
    let sql = "INSERT INTO notes (title, content) VALUES (?1, ?2)";
    conn.execute(sql, (title.trim(), content.trim()))?;
    
    /* I understand that the input handler already returned a trimmed String on the heap but I'm doing this for the best practice that if in future the io_handler is revised and something breaks, the sql point of insertion will be safe.
     I could alternatively do:
    conn.execute(sql, (&title, &content))?;
    Actually, .trim() helps converts the String Type to the string literal based on how it works under the hood. One stone, two birds killed.
     */
    println!("Note created successfully.");
    Ok(())
}


