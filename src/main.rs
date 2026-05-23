<<<<<<< HEAD
use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "notebook")]
#[command(about = "A simple CLI notebook for taking notes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new note
    Add {
        #[arg(short, long)]
        title: String,

        #[arg(short, long)]
        content: String,

        #[arg(short, long)]
        tags: Option<String>,
    },

    /// List all notes
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Show only recent N notes
        #[arg(short, long)]
        recent: Option<usize>,
    },

    /// View a specific note
    View { id: i32 },

    /// Delete a note
    Delete {
        id: i32,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Search notes by content or title
    Search { query: String },

    /// Edit an existing note
    Edit {
        id: i32,

        #[arg(short, long)]
        title: Option<String>,

        #[arg(short, long)]
        content: Option<String>,
    },
}

#[derive(Debug)]
struct Note {
    id: i32,
    title: String,
    content: String,
    tags: String,
    created_at: String,
    updated_at: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;
    initialize_database(&conn)?;

    match cli.command {
        Commands::Add { title, content, tags } => add_note(&conn, title, content, tags)?,
        Commands::List { tag, recent } => list_notes(&conn, tag, recent)?,
        Commands::View { id } => view_note(&conn, id)?,
        Commands::Delete { id, force } => delete_note(&conn, id, force)?,
        Commands::Search { query } => search_notes(&conn, &query)?,
        Commands::Edit { id, title, content } => edit_note(&conn, id, title, content)?,
    }

    Ok(())
}

fn get_db_path() -> Result<String> {
    let home = std::env::var("HOME").context("Could not find home directory")?;
    let notebook_dir = PathBuf::from(&home).join(".notebook");

    if !notebook_dir.exists() {
        std::fs::create_dir_all(&notebook_dir)?;
    }

    Ok(notebook_dir
        .join("notes.db")
        .to_str()
        .context("Invalid database path")?
        .to_string())
}

fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_tags ON notes(tags)", [])?;

    Ok(())
}

fn add_note(conn: &Connection, title: String, content: String, tags: Option<String>) -> Result<()> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tags_str = tags.unwrap_or_default();

    conn.execute(
        "INSERT INTO notes (title, content, tags, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![title, content, tags_str, now, now],
    )?;

    let last_id = conn.last_insert_rowid();
    println!("✓ Note added successfully! (ID: {})", last_id);

    Ok(())
}

fn list_notes(conn: &Connection, tag: Option<String>, recent: Option<usize>) -> Result<()> {
    let mut query = String::from("SELECT id, title, tags, created_at FROM notes");
    let mut params: Vec<String> = Vec::new();

    if let Some(ref tag_filter) = tag {
        query.push_str(" WHERE tags LIKE ?1");
        params.push(format!("%{}%", tag_filter));
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = recent {
        query.push_str(" LIMIT ?");
        params.push(limit.to_string());
    }

    let mut stmt = conn.prepare(&query)?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params
        .iter()
        .map(|p| p as &dyn rusqlite::ToSql)
        .collect();

    let note_iter = stmt.query_map(&*param_refs, |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: String::new(), // not used in list view
            tags: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: String::new(), // not used in list view
        })
    })?;

    let notes: Vec<Note> = note_iter.filter_map(Result::ok).collect();

    if notes.is_empty() {
        println!("× No notes found.");
        if tag.is_some() {
            println!("! Try removing the tag filter.");
        }
    } else {
        println!("✓ Your Notes ({} total):\n", notes.len());
        for note in notes {
            println!("[{:3}] {}", note.id, note.title);
            if !note.tags.is_empty() {
                println!("      Tags: {}", note.tags);
            }
            println!("      Created: {}\n", note.created_at);
=======
mod io_handler;
mod delete_note;
mod create_note;
mod print_menu; 
mod read_notes;

use rusqlite::{Connection, Result, Row};
use std::error::Error;

use crate::{
    delete_note::delete_note,
    create_note::create_note,
    io_handler::io_handler,
    print_menu::print_menu,
    read_notes::read_notes
};

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open("notebook.db")?;

    // Create table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
// Do we manually handle timestamp?
    
    println!("      NOTEBOOK APPLICATION STARTED.     ");
    print_menu();
    
    loop {
        
        let choice = io_handler("--- Enter your choice Number");

        match choice.trim() {
            "0" => print_menu(),
            "1" => create_note(&conn)?,
            "2" => read_notes(&conn)?, // KEEP AN EYE
       //     "3" => update_note(&conn)?,
            "4" => delete_note(&conn)?,
            "5" => {
                println!("Exiting application. Goodbye.");
                break;
            }
            _ => println!("Invalid! Please try again with a NUMBER from the menu."),
>>>>>>> 259522b (Separation Of Concerns)
        }
    }

    Ok(())
}

<<<<<<< HEAD
fn view_note(conn: &Connection, id: i32) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, tags, created_at, updated_at FROM notes WHERE id = ?1",
    )?;

    let note = stmt.query_row(params![id], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            tags: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    });

    match note {
        Ok(note) => {
            println!("\n Note #{}\n", note.id);
            println!("Title: {}", note.title);
            println!("Created: {}", note.created_at);
            println!("Updated: {}", note.updated_at);
            if !note.tags.is_empty() {
                println!("Tags: {}", note.tags);
            }
            println!("\n---\n{}\n", note.content);
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            println!("× Note with ID {} not found.", id);
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

fn delete_note(conn: &Connection, id: i32, force: bool) -> Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM notes WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;

    if !exists {
        println!("× Note with ID {} not found.", id);
        return Ok(());
    }

    if !force {
        println!("? Are you sure you want to delete note #{}? [y/N]", id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Delete cancelled.");
            return Ok(());
        }
    }

    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;

    if affected > 0 {
        println!("✓ Note #{} deleted successfully.", id);
    }

    Ok(())
}

fn search_notes(conn: &Connection, query: &str) -> Result<()> {
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, title, content, tags, created_at 
         FROM notes 
         WHERE title LIKE ?1 OR content LIKE ?2 OR tags LIKE ?3
         ORDER BY created_at DESC",
    )?;

    let note_iter = stmt.query_map(
        params![search_pattern, search_pattern, search_pattern],
        |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: String::new(),
            })
        },
    )?;

    let notes: Vec<Note> = note_iter.filter_map(Result::ok).collect();

    if notes.is_empty() {
        println!("× No notes found matching '{}'", query);
    } else {
        println!("✓ Found {} note(s) matching '{}':\n", notes.len(), query);
        for note in notes {
            println!("[{:3}] {}", note.id, note.title);
            let preview = if note.content.len() > 60 {
                format!("{}...",
                    note.content[..60].to_string())
            } else {
                note.content
            };
            println!("      Preview: {}", preview);
            if !note.tags.is_empty() {
                println!("      Tags: {}", note.tags);
            }
            println!();
        }
    }

    Ok(())
}

fn edit_note(
    conn: &Connection,
    id: i32,
    new_title: Option<String>,
    new_content: Option<String>,
) -> Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM notes WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;

    if !exists {
        println!("× Note with ID {} not found.", id);
        return Ok(());
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    match (new_title, new_content) {
        (Some(title), Some(content)) => {
            conn.execute(
                "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                params![title, content, now, id],
            )?;
            println!("✓ Note #{} updated completely.", id);
        }
        (Some(title), None) => {
            conn.execute(
                "UPDATE notes SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, now, id],
            )?;
            println!("✓ Note #{} title updated.", id);
        }
        (None, Some(content)) => {
            conn.execute(
                "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, now, id],
            )?;
            println!("✓ Note #{} content updated.", id);
        }
        (None, None) => {
            println!("No changes provided. Use --title or --content to update.");
        }
    }

    Ok(())
}
=======

/*
fn update_note(conn: &Connection) -> Result<()> {
  //  read_notes(conn)?; // Show existing notes first

    let id_str = io_handler("Enter the ID of the note to update");
    let id: i32 = match id_str.trim().parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid ID format.");
            return Ok(());
        }
    };

    let title = io_handler("Enter new title (leave empty to keep current)");
    let content = io_handler("Enter new content (leave empty to keep current)");

    let mut updates = vec![];
    let mut params: Vec<&dyn rusqlite::ToSql> = vec![];

    if !title.trim().is_empty() {
        updates.push("title = ?1");
        params.push(&title.trim());
    }
    if !content.trim().is_empty() {
        updates.push("content = ?2");
        params.push(&content.trim());
    }

    if updates.is_empty() {
        println!("No changes provided.");
        return Ok(());
    }

    let sql = format!(
        "UPDATE notes SET {} WHERE id = ?{}",
        updates.join(", "),
        if updates.len() == 1 { "2" } else { "3" }
    );

    params.push(&id);

    let affected = conn.execute(&sql, rusqlite::params_from_iter(params))?;
    
    if affected > 0 {
        println!("Note updated successfully.");
    } else {
        println!("No note found with ID: {}", id);
    }
    Ok(())
}
*/


>>>>>>> 259522b (Separation Of Concerns)
