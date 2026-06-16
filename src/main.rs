// ============================================================
// We welcome you to our orange-flavored CLI notebook!
// We store every note in a local SQLite database so our data
// survives between sessions without any server nonsense.
// ============================================================

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use std::path::PathBuf;

// ── We define our orange palette with raw ANSI escape codes ──
// This keeps us dependency-free while still looking gorgeous.

// const ORANGE: &str = "\x1b[38;5;214m"; // Our signature orange
const ORANGE_BOLD: &str = "\x1b[1;38;5;214m"; // Bold orange for headers
const AMBER: &str = "\x1b[38;5;208m"; // Slightly darker for accents
const DIM: &str = "\x1b[38;5;180m"; // Muted warm tone for metadata
const WHITE_BOLD: &str = "\x1b[1;97m"; // Bright white for note bodies
const RED: &str = "\x1b[38;5;203m"; // We use red only for errors
const GREEN: &str = "\x1b[38;5;118m"; // We use green for success
const RESET: &str = "\x1b[0m"; // We always reset at the end

// ── We define the shape of a note as it lives in our database ──
#[derive(Debug)]
struct Note {
    id: i64,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
}

// ── We use clap's derive macro to give us a beautiful CLI for free ──
#[derive(Parser)]
#[command(
    name = "notebook",
    about = "📓 Our orange-powered terminal notebook",
    version = "0.1.0",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

// We enumerate every action our notebook understands.
#[derive(Subcommand)]
enum Command {
    /// We add a brand-new note
    Add {
        /// The title we give the note
        title: String,
        /// The body content of our note
        content: String,
    },

    /// We list every note we have stored
    List,

    /// We display a single note in full
    View {
        /// The ID of the note we want to see
        id: i64,
    },

    /// We update an existing note's content (and optionally its title)
    Edit {
        /// The ID of the note we want to change
        id: i64,
        /// The new content body we are writing
        content: String,
        /// An optional new title — we keep the old one if omitted
        #[arg(short, long)]
        title: Option<String>,
    },

    /// We permanently delete a note — no undo!
    Delete {
        /// The ID of the note we wish to remove
        id: i64,
    },

    /// We search through all our notes for a keyword
    Search {
        /// The search term we are hunting for
        query: String,
    },
}

// ── We resolve where our database file lives on disk ──
// We follow the XDG convention: ~/.local/share/notebook/notes.db
fn db_path() -> Result<PathBuf> {
    // We prefer the user's data directory; we fall back to home.
    let base = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let dir = base.join(".local").join("share").join("notebook");

    // We create the directory if it doesn't exist yet.
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("We couldn't create our data directory: {dir:?}"))?;

    Ok(dir.join("notes.db"))
}

// ── We open (or create) our SQLite connection ──
fn open_db() -> Result<Connection> {
    let path = db_path()?;
    let conn = Connection::open(&path)
        .with_context(|| format!("We couldn't open our database at {path:?}"))?;

    // We enable WAL mode so our writes don't block reads.
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    Ok(conn)
}

// ── We run our initial migration the first time we start up ──
fn init_schema(conn: &Connection) -> Result<()> {
    // We create the notes table only if it isn't there yet —
    // this keeps us safe to call on every single startup.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            title      TEXT    NOT NULL,
            content    TEXT    NOT NULL,
            created_at TEXT    NOT NULL,
            updated_at TEXT    NOT NULL
        );",
    )?;
    Ok(())
}

// ── We produce a timestamp string in our preferred format ──
fn now() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ── We print our fancy orange banner at the top of list/search output ──
fn print_banner(label: &str) {
    println!("\n{ORANGE_BOLD}╔══════════════════════════════════════╗{RESET}");
    println!("{ORANGE_BOLD}║  📓  {WHITE_BOLD}{label:<32}{ORANGE_BOLD}║{RESET}");
    println!("{ORANGE_BOLD}╚══════════════════════════════════════╝{RESET}\n");
}

// ── We draw a horizontal rule to separate notes visually ──
fn divider() {
    println!("{AMBER}──────────────────────────────────────────{RESET}");
}

// ────────────────────────────────────────────────────────────
// We implement each subcommand below.
// ────────────────────────────────────────────────────────────

// We add a note and echo back its freshly assigned ID.
fn cmd_add(conn: &Connection, title: &str, content: &str) -> Result<()> {
    let ts = now();
    conn.execute(
        "INSERT INTO notes (title, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![title, content, ts, ts],
    )?;

    let id = conn.last_insert_rowid();

    println!("\n{GREEN}✔  Note saved!{RESET}  {DIM}id = {ORANGE_BOLD}{id}{RESET}\n");
    Ok(())
}

// We fetch every note and render a compact summary table.
fn cmd_list(conn: &Connection) -> Result<()> {
    let mut stmt =
        conn.prepare("SELECT id, title, created_at, updated_at FROM notes ORDER BY id ASC")?;

    // We collect the rows first so we can report "nothing here yet".
    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;

    if rows.is_empty() {
        println!(
            "\n{AMBER}  We don't have any notes yet. Try:{RESET}\n\
             {DIM}    notebook add \"My Title\" \"My content\"{RESET}\n"
        );
        return Ok(());
    }

    print_banner("Our Notes");

    for (id, title, created, updated) in &rows {
        println!("  {ORANGE_BOLD}[{id:>4}]{RESET}  {WHITE_BOLD}{title}{RESET}");
        println!("         {DIM}created {created}  ·  updated {updated}{RESET}");
        println!();
    }

    println!(
        "{DIM}  We have {}{}{DIM} note(s) in our notebook.{RESET}\n",
        ORANGE_BOLD,
        rows.len(),
    );

    Ok(())
}

// We look up a single note by ID and display it in full.
fn cmd_view(conn: &Connection, id: i64) -> Result<()> {
    let result = conn.query_row(
        "SELECT id, title, content, created_at, updated_at FROM notes WHERE id = ?1",
        params![id],
        |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(note) => {
            println!();
            divider();
            println!("{ORANGE_BOLD}  #{} — {}{RESET}", note.id, note.title);
            divider();
            println!("{WHITE_BOLD}\n{}{RESET}", note.content);
            println!();
            println!("{DIM}  Created : {}{RESET}", note.created_at);
            println!("{DIM}  Updated : {}{RESET}", note.updated_at);
            divider();
            println!();
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // We inform the user calmly — there's no note with that ID.
            eprintln!("\n{RED}  ✘  We couldn't find a note with id = {id}.{RESET}\n");
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

// We overwrite the content (and optionally the title) of an existing note.
fn cmd_edit(conn: &Connection, id: i64, new_content: &str, new_title: Option<&str>) -> Result<()> {
    let ts = now();

    // We check whether the note actually exists before we try to update it.
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    if count == 0 {
        eprintln!("\n{RED}  ✘  We couldn't find a note with id = {id}.{RESET}\n");
        return Ok(());
    }

    if let Some(title) = new_title {
        // We update both the title and the content in one shot.
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![title, new_content, ts, id],
        )?;
    } else {
        // We leave the title untouched and only update the content.
        conn.execute(
            "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_content, ts, id],
        )?;
    }

    println!("\n{GREEN}✔  Note {ORANGE_BOLD}{id}{GREEN} updated successfully.{RESET}\n");

    Ok(())
}

// We delete a note — we keep it simple and permanent.
fn cmd_delete(conn: &Connection, id: i64) -> Result<()> {
    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;

    if affected == 0 {
        // We let the user know we couldn't find anything to delete.
        eprintln!("\n{RED}  ✘  We couldn't find a note with id = {id}.{RESET}\n");
    } else {
        println!("\n{GREEN}✔  Note {ORANGE_BOLD}{id}{GREEN} deleted.{RESET}\n");
    }

    Ok(())
}

// We search titles and content with SQLite's LIKE operator.
fn cmd_search(conn: &Connection, query: &str) -> Result<()> {
    // We wrap the query in wildcards so partial matches work too.
    let pattern = format!("%{query}%");

    let mut stmt = conn.prepare(
        "SELECT id, title, content, created_at, updated_at
         FROM   notes
         WHERE  title   LIKE ?1
            OR  content LIKE ?1
         ORDER BY id ASC",
    )?;

    let rows: Vec<Note> = stmt
        .query_map(params![pattern], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;

    if rows.is_empty() {
        println!("\n{AMBER}  We found no notes matching {query}.{RESET}\n");
        return Ok(());
    }

    print_banner(&format!("Results for \"{query}\""));

    for note in &rows {
        println!(
            "  {ORANGE_BOLD}[{:>4}]{RESET}  {WHITE_BOLD}{}{RESET}",
            note.id, note.title
        );

        // We show a snippet of the content so the user knows why it matched.
        let snippet: String = note.content.chars().take(80).collect();
        let ellipsis = if note.content.len() > 80 { "…" } else { "" };
        println!("         {DIM}{snippet}{ellipsis}{RESET}");
        println!();
    }

    println!(
        "{DIM}  We matched {}{}{DIM} note(s).{RESET}\n",
        ORANGE_BOLD,
        rows.len(),
    );

    Ok(())
}

// ── Our entry point — we tie everything together here ──
fn main() -> Result<()> {
    // We parse the arguments first so clap can print --help on bad input.
    let cli = Cli::parse();

    // We open our database and ensure our schema is ready.
    let conn = open_db()?;
    init_schema(&conn)?;

    // We dispatch to whichever subcommand the user requested.
    match &cli.command {
        Command::Add { title, content } => {
            cmd_add(&conn, title, content)?;
        }
        Command::List => {
            cmd_list(&conn)?;
        }
        Command::View { id } => {
            cmd_view(&conn, *id)?;
        }
        Command::Edit { id, content, title } => {
            cmd_edit(&conn, *id, content, title.as_deref())?;
        }
        Command::Delete { id } => {
            cmd_delete(&conn, *id)?;
        }
        Command::Search { query } => {
            cmd_search(&conn, query)?;
        }
    }

    Ok(())
}
