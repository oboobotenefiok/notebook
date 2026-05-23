use crate::io_handler::io_handler;
use rusqlite::{Connection, Result, Row};

pub fn delete_note(conn: &Connection) -> Result<()> {
   // read_notes(conn)?; // Show existing notes first

    let id_str = io_handler("Enter the ID of the note to delete");
    let id: i32 = match id_str.trim().parse() {
        // This is me double-trimming again!
        Ok(n) => n,
        Err(_) => {
            println!("Invalid ID format.");
            return Ok(()); //This is a problem?
        }
    };

    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", (id,))?;
// Hehehe
    if affected > 0 {
        println!("Note deleted successfully.");
    } else {
        println!("No note found with ID: {}", id);
    }
    Ok(())
}