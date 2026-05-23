use rusqlite::{ Connection, Result, Row};

pub fn read_notes(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, title, content FROM notes")?;
    
    /*  created_at */
    let notes_iter = stmt.query_map([], |row: &Row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
          //  row.get::<_, String>(3)?,
        ))
    })?;

    println!("\n--- All Notes ---");
    let mut count = 0;
    for note in notes_iter {
        let (id, title, content, /*time*/) = note?;
        println!("ID: {}", id);
        println!("Title: {}", title);
        println!("Content: {}", content);
     //   println!("Created: {}", time);
        println!("-------------------");
        count += 1;
    }

    if count == 0 {
        println!("No notes found.");
    } else {
        println!("Total notes: {}", count);
    }
    Ok(())
}
