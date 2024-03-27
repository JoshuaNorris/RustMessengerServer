#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate rusqlite;
extern crate time;

use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Result};
use time::Timespec;
use std::{io, fmt};
use std::collections::LinkedList;
use crate::CONN;
use std::sync::{Arc, Mutex};
use std::ffi::CString;
use std::error::Error;


#[derive(Debug)]
pub struct Messages {
    pub from_user: String,
    pub to_user: String,
    pub have_read: u8,
    pub contents: String
}

impl fmt::Display for Messages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.from_user, self.to_user, self.contents)
    }
}

impl Messages {
    // TESTED -- test_new_message()
    fn new(from_user: String, to_user: String, contents: String) -> Messages {
        // This function creates a message object with the username of the user who sent
        // the message and the username of the user who the message was intended for
        // and the message itself.
        // This function is only called in the parse_messages function at the bottom of this class.

        Messages {
            from_user,
            to_user,
            have_read: 0, // We assume that a new message has not been read by the other user.
            contents
        }
    }

    // TESTED -- test_database()
    pub fn insert(self) -> Result<()> {
        // This function inserts the current Messages object into the database.
        let conn = CONN.lock().expect("Could not lock CONN.");
        conn.execute(
            "INSERT INTO Messages (from_user, to_user, have_read, contents)
                VALUES (?1, ?2, ?3, ?4);",
            params![self.from_user, self.to_user, self.have_read, self.contents],
        ).expect("There is something wrong with the SQL Insert statement.");

        Ok(())
    }
}

// TESTED -- test_database()
pub fn get_conversation(from_user: String) -> Result<String> {
    // This function returns all of the messages that have occurred so far between two users.
    // It considers both when the from_user was the from_user and to_user and vice-versa.
    // It also orders it with the most recent messages last.

    let conn = CONN.lock().expect("Could not lock CONN.");

    let mut stmt = conn.prepare(
        "SELECT * FROM Messages
            WHERE (from_user=?1 OR to_user=?1) AND have_read=0
            ORDER BY id ASC;")
        .expect("The Select statement is wrong.");
    let message_iterate = stmt.query_map(params![from_user], |row| {
        Ok(Messages {
            from_user: row.get(1).expect("Could not get from_user from database."),
            to_user: row.get(2).expect("Could not get to_user from database."),
            have_read: row.get(3).expect("Could not get have_read from database."),
            contents: row.get(4).expect("Could not get contents from database."),
        })
    })?;
    let mut result:String = String::new();
    for message in message_iterate {
        result = format!("{}\n{}", result, message.expect("Could not push messages."));
    }

    // Below line gets rid of an unwanted newline at the start of the string.
    result = String::from(result.trim_left_matches('\n'));
    conn.execute(
        "UPDATE Messages
                SET have_read = 1
                WHERE (from_user=?1 OR to_user=?1) AND have_read=0;",
        params!(from_user))
        .expect("The Update statement is wrong.");
    Ok(result)

}


// TESTED -- test_database()
pub fn parse_message(message: String) -> Messages {
    // Given the POST command this returns a messages object for the message that was sent.
    // This objects contains who the message was from, who it was to, if it has been read,
    // and the contents of the message.

    let mut split_message = message.split_whitespace();
    split_message.next(); // This is for the POST portion of the command.
    let from_user = String::from(split_message.next().unwrap());
    let to_user = String::from(split_message.next().unwrap());

    // Below is gathering the rest of the words as content of the message
    let mut next_word = split_message.next();
    let mut contents = String::new();
    while next_word.is_some() {
        contents = format!("{} {}", contents, next_word.unwrap());
        next_word = split_message.next();
    }
    // Above is gathering the rest of the words as content of the message

    // Below line gets rid of an unwanted space at the start of the string.
    contents = String::from(contents.trim_left());
    Messages::new(from_user, to_user, contents)
}

// TESTED -- test_database()
pub fn init_messages() -> Result<()> {
    // This initializes the database.
    // This should only be called once in the existence of the server
    // Whenever the server shuts down this function should not need to be called again.

    // This also completely erases a database if that is ever needed for testing purposes.

    let conn = CONN.lock().expect("Could not lock CONN.");
    conn.execute(
        "DROP TABLE Messages",
        params![]
    );
    conn.execute(
        "CREATE TABLE Messages (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            from_user       TEXT NOT NULL,
            to_user			TEXT NOT NULL,
            have_read       INTEGER NOT NULL,
            contents        TEXT NOT NULL);",
        params!()).expect("The create table statement is wrong.");
    Ok(())
}





#[test]
fn test_new_message() {
    // This just tests the new() function for Messages.
    let message = Messages {
        from_user: String::from("JOSH"),
        to_user: String::from("ALI"),
        have_read: 0 as u8, // 0 is the same as false in SQL
        contents: String::from("HELLO"),
    };
    let other_message = Messages::new(String::from("JOSH"), String::from("ALI"), String::from("HELLO"));
    assert_eq!(other_message.from_user, message.from_user);
    assert_eq!(other_message.to_user, message.to_user);
    assert_eq!(other_message.have_read, message.have_read);
    assert_eq!(other_message.contents, message.contents);
}

#[test]
fn test_database() {
    // This tests the creation, insertion into, and querying from the database. Oh vey.

    init_messages();
    let message1 = String::from("POST JOSH RACHAEL HELLO");
    let message2 = String::from("POST RACHAEL JOSH HOWARE YOU");

    let message1 = parse_message(message1);
    let message2 = parse_message(message2);
    message1.insert();
    message2.insert();

    let conversation = get_conversation(String::from("JOSH"));

    let mut answer = String::from("JOSH RACHAEL HELLO\nRACHAEL JOSH HOWARE YOU");

    assert_eq!(conversation.expect("Problem with conversation."), answer);

    let message1 = String::from("POST JOSH RACHAEL I AM GOOD");
    let message2 = String::from("POST RACHAEL JOSH I AM LEAVING");

    let message1 = parse_message(message1);
    let message2 = parse_message(message2);
    message1.insert();
    message2.insert();

    let mut answer = String::from("JOSH RACHAEL I AM GOOD\nRACHAEL JOSH I AM LEAVING");

    // This last line removes the contents from the database, so the data does not stay for when
    // the database is actually used.
    init_messages();
}