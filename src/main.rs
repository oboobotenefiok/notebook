

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{Connection, params};
use std::path::Path;


#[derive(Parser)]
#[command(name = "notebook")]
#[command(about = "A simple CLI notebook for taking notes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    
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
    View {
        /// Note ID
        id: i32,
    },
    
    /// Delete a note
    Delete {
        /// Note ID
        id: i32,
        
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Search notes by content or title
    Search {
        /// Search query
        query: String,
    },
    
    /// Edit an existing note
    Edit {
        /// Note ID
        id: i32,
        
        /// New title (optional)
        #[arg(short, long)]
        title: Option<String>,
        
        /// New content (optional)
        #[arg(short, long)]
        content: Option<String>,
    },
}

// Note structure
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
        Commands::Add { title, content, tags } => {
            add_note(&conn, title, content, tags)?;
        }
        Commands::List { tag, recent } => {
            list_notes(&conn, tag, recent)?;
        }
        Commands::View { id } => {
            view_note(&conn, id)?;
        }
        Commands::Delete { id, force } => {
            delete_note(&conn, id, force)?;
        }
        Commands::Search { query } => {
            search_notes(&conn, &query)?;
        }
        Commands::Edit { id, title, content } => {
            edit_note(&conn, id, title, content)?;
        }
    }
    
    Ok(())
}

// Get database path (creates .notebook directory in home)
fn get_db_path() -> Result<String> {
    let home = std::env::var("HOME").context("Could not find home directory")?;
    let notebook_dir = Path::new(&home).join(".notebook");
    
    if !notebook_dir.exists() {
        std::fs::create_dir_all(&notebook_dir)?;
    }
    
    Ok(notebook_dir.join("notes.db").to_str().unwrap().to_string())
}

// Initialize database schema
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
    
    // Create index on tags for faster searching
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tags ON notes(tags)",
        [],
    )?;
    
    Ok(())
}

// Add a new note
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

// List all notes
fn list_notes(conn: &Connection, tag: Option<String>, recent: Option<usize>) -> Result<()> {
    let mut query = String::from(
        "SELECT id, title, tags, created_at FROM notes"
    );
    let mut params: Vec<String> = vec![];
    
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
            content: String::new(), // Not needed for list view
            tags: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: String::new(),
        })
    })?;
    
    let notes: Vec<Note> = note_iter.filter_map(Result::ok).collect();
    
    if notes.is_empty() {
        println!("× No notes found.");
        if tag.is_some() {
            println!("! Try removing the tag filter.")else {
        println!("✓ Your Notes ({} total):\n", notes.len());
        for note in notes {
            println!("[{:3}] {}", note.id, note.title);
            if !note.tags.is_empty() {
                println!("      Tags: {}", note.tags);
            }
            println!("      Created: {}\n", note.created_at);
        }
    }
    
    Ok(())
}

// View a specific note
fn view_note(conn: &Connection, id: i32) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, tags, created_at, updated_at FROM notes WHERE id = ?1"
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

// Delete a note
fn delete_note(conn: &Connection, id: i32, force: bool) -> Result<()> {
    // First check if note exists
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

// Search notes
fn search_notes(conn: &Connection, query: &str) -> Result<()> {
    let search_pattern = format!("%{}%", query);
    
    let mut stmt = conn.prepare(
        "SELECT id, title, content, tags, created_at 
         FROM notes 
         WHERE title LIKE ?1 OR content LIKE ?2 OR tags LIKE ?3
         ORDER BY created_at DESC"
    )?;
    
    let note_iter = stmt.query_map(params![search_pattern, search_pattern, search_pattern], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            tags: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: String::new(),
        })
    })?;
    
    let notes: Vec<Note> = note_iter.filter_map(Result::ok).collect();
    
    if notes.is_empty() {
        println!("× No notes found matching '{}'", query);
    } else {
        println!("✓ Found {} note(s) matching '{}':\n", notes.len(), query);
        for note in notes {
            println!("[{:3}] {}", note.id, note.title);
            println!("      Preview: {}", 
                if note.content.len() > 60 {
                    format!("{}...", &note.content[..60])
                } else {
                    note.content
                }
            );
            if !note.tags.is_empty() {
                println!("      Tags: {}", note.tags);
            }
            println!();
        }
    }
    
    Ok(())
}

// Edit an existing note
fn edit_note(conn: &Connection, id: i32, new_title: Option<String>, new_content: Option<String>) -> Result<()> {
    // Check if note exists
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
    
    if let (Some(title), Some(content)) = (new_title, new_content) {
        // Update both
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![title, content, now, id],
        )?;
        println!("✓ Note #{} updated completely.", id);
    } else if let Some(title) = new_title {
        // Update only title
        conn.execute(
            "UPDATE notes SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )?;
        println!("✓ Note #{} title updated.", id);
    } else if let Some(content) = new_content {
        // Update only content
        conn.execute(
            "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![content, now, id],
        )?;
        println!("✓ Note #{} content updated.", id);
    } else {
        println!("No changes provided. Use --title or --content to update.");
    }
    
    Ok(())
}
        
        
        //I'll now work on modularizing the codebase
        // And correct the few mistakes