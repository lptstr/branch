use std::result;
use std::fs;
use std::io;
use std::env;

use std::vec::Vec;
use std::path::Path;
use clap::ArgMatches;
use rayon::prelude::*;

use crate::file::*;
use crate::outp::*;

const E: char = 27 as char;

const BRANCH_ENTRY_STR: &str        = "├── ";
const BRANCH_LINE_STR: &str         = "│   ";
const BRANCH_LASTENTRY_STR: &str    = "└── ";
const BRANCH_BLANK_STR: &str        = "    ";

// get listing of contents of this
// directory
fn tree(
            matches: &ArgMatches, 
            directory: &str, 
            prefix: &str, 
            mut treestat: &mut TreeStatistics
       ) -> result::Result<(), io::Error> 
{
    // walk file tree
    let mut things: Vec<_> = fs::read_dir(&*directory)?.map(|thing| {
        thing.unwrap().path()
    }).collect();
    let mut index = things.len();

    // don't sort if told not to
    if ! matches.is_present("nosort") {
        // sort these paths via rayon
        things.par_sort_unstable_by(|a, b| {
            let aname = a.file_name().unwrap().to_str().unwrap();
            let bname = b.file_name().unwrap().to_str().unwrap();
            aname.cmp(bname)
        });
    }

    // iter over paths and display 
    for thing in things {
        // skip this thing if it's hidden and --all is not set
        if ! matches.is_present("all") {
            if is_hidden(&thing) {
                continue;
            }
        }
        
        let is_dir: bool = thing.is_dir();
        let thing = thing.file_name().unwrap().to_str().unwrap();
        index = index - 1;

        // customize this iteration's str
        let mut current_branch_str;
        if index == 0 {
            current_branch_str = BRANCH_LASTENTRY_STR;
        } else {
            current_branch_str = BRANCH_ENTRY_STR;
        }
        
        // increment tree statistics
        if is_dir {
            treestat.directories += 1;
        } else {
            treestat.files += 1
        }

        // display
        //if matches.is_present("dirs") && is_dir || ! matches.is_present("dirs") && ! is_dir {
            println!("{}{}{}", prefix, current_branch_str, thing);
        //}

        // check if path is directory, and if so, 
        // recursively get contents
        if is_dir {
            // use rayon to (possibly) execute this task in parallel
            rayon::scope(|s| {
                s.spawn(|_| {
                    tree(
                        matches,
                        &format!("{}/{}", directory, thing), 
                        &format!("{}{}", prefix, BRANCH_LINE_STR), 
                        &mut treestat).unwrap();
                });
            });
        }
        debug(format!("found entry {}", thing));
    }
    debug("done".to_owned());
    Ok(())
}

pub fn branch(matches: &ArgMatches) {
    let mut directory: String = env::current_dir()
        .unwrap()
        .display()
        .to_string();

    // get directory
    if let Some(dir) = matches.value_of("PATH") {
        directory = dir.to_string();
    }

    // check that directory exists
    if ! fs::metadata(&directory).is_ok() {
        die(format!("directory {} does not exist.", directory));
    }

    // check that the thing is a directory 
    if ! Path::new(&directory).is_dir() {
        die(format!("path {} isn't a directory.", directory));
    }

    // add / to path
    if directory.chars().last().unwrap() != '/' {
        directory = directory + "/"
    }

    // print directory
    println!("{}[1;34m{}{}[0m\n", E, directory, E);

    // init tree statistics
    let mut treestat = TreeStatistics { directories: 0, files: 0 };

    // print everything
    let result = tree(matches, &directory.clone(), "", &mut treestat);

    // match errors, just in case
    match result {
        Ok(()) => println!("\n{} directories, {} files", treestat.directories, treestat.files),
        Err(err) => error(format!(" {:?}", err)),
    }
}
