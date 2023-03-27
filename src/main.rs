use scraper::{Html, Selector};
use std::io;

const WIKI_ROOT: &str = "https://azurlane.koumakan.jp";

fn main() {
    let cv_cvl_list = get_cv_cvl_list();
    println!("Found {} results, processing ...", cv_cvl_list.len());

    // let cv_cvl_list = vec![
    //     ("Unicorn".to_string(), "/wiki/Unicorn".to_string()),
    // ];

    let mut result: Vec<Vec<String>> = Vec::new();

    for ship in cv_cvl_list {
        result.push(get_cv_cvl_data(ship));
    }

    export_csv(result)
}

fn get_cv_cvl_list() -> Vec<(String, String)> {

    let response = reqwest::blocking::get(format!("{}/wiki/List_of_Ships_by_Stats", WIKI_ROOT))
        .expect("invalid wiki list url")
        .text()
        .expect("invalid wiki list contents");

    let document = Html::parse_document(&response);

    let section_selector = Selector::parse("[class='tabber']").unwrap();
    let cv_cvl_section = document
        .select(&section_selector)
        .nth(4)
        .expect("unable to find the 5th cv table");

    let table_selector = Selector::parse("[class='azltable sortable']").unwrap();
    let cv_cvl_table = cv_cvl_section
        .select(&table_selector)
        .next()
        .expect("unable to find the inner cv table");

    let row_selector = Selector::parse("tr").unwrap();
    let rows = cv_cvl_table
        .select(&row_selector)
        .skip(1)
        .collect::<Vec<_>>();

    let mut data: Vec<(String, String)> = Vec::new();

    for row in rows {
        // sometimes we have empty rows, and we need to skip those
        if row.value().attr("class").unwrap_or_default() == "mw-empty-elt"{
            continue
        }

        let column_selector = Selector::parse("td").unwrap();

        let column_data = row
            .select(&column_selector)
            .nth(1)
            .expect("Unable to find the second column with the cell");

        let cell_selector = Selector::parse("a").unwrap();
        let cell_data = column_data
            .select(&cell_selector)
            .next()
            .expect("unable to find the a selector with the href link");

        let name = cell_data
            .text()
            .nth(0)
            .expect("unable to find the name");

        let link = cell_data
            .value()
            .attr("href")
            .expect("unable to find the link");

        data.push((name.to_string(), link.to_string()))
    }

    data
}

fn get_cv_cvl_data(ship: (String, String)) -> Vec<String> {

    let response = reqwest::blocking::get(format!("{}{}", WIKI_ROOT, ship.1))
        .expect(format!("invalid ship list url: {}", ship.1).as_str())
        .text()
        .expect(format!("invalid ship list contents: {}", ship.1).as_str());

    let document = Html::parse_document(&response);

    let table_selector = Selector::parse("[class='ship-equipment wikitable'] tbody").unwrap();
    let table_section = document
        .select(&table_selector)
        .nth(0)
        .expect("unable to find the equipment table");

    let row_selector = Selector::parse("tr").unwrap();
    let all_rows = table_section
        .select(&row_selector)
        .collect::<Vec<_>>();

    let rows = all_rows[2..5].to_vec();

    // index meanints
    // 0: fighters
    // 1: dive bombers
    // 2. torp bombers
    // 3. anti-air
    // 3. other
    let mut equipment: Vec<u8> = vec![0, 0, 0, 0, 0];

    for row in rows {
        let column_selector = Selector::parse("td").unwrap();
        let columns = row
            .select(&column_selector)
            .collect::<Vec<_>>();

        let equipment_name = columns[2]
            .text()
            .collect::<Vec<_>>()
            .join(" ");

        let equipment_match = columns[3]
            .text()
            .last()
            .expect("cannot find equipment count")
            .trim()
            .parse::<u8>();

        let equipment_count = match equipment_match {
            Ok(v) => v,
            Err(e) => {
                let retrofit_selector = Selector::parse("span").unwrap();
                columns[3]
                    .select(&retrofit_selector)
                    .nth(0)
                    .expect("Cannot find retrofit selector")
                    .text()
                    .last()
                    .expect("cannot find retrofit text")
                    .trim()
                    .parse::<u8>()
                    .expect("Expect integer as equipment count")
            }
        };

        match equipment_name.trim().to_ascii_lowercase().as_str() {
            "fighters" => equipment[0] += equipment_count,
            "dive bombers" => equipment[1] += equipment_count,
            "torpedo bombers" => equipment[2] += equipment_count,
            "anti-air guns" => equipment[3] += equipment_count,
            _ => equipment[4] += equipment_count,
        }
    }

    let mut temp: Vec<String> = equipment.into_iter().map(|n| n.to_string()).collect();
    temp.push(ship.0);

    temp
}

fn export_csv(input: Vec<Vec<String>>){
    let mut writer = csv::Writer::from_writer(io::stdout());

    for lines in input{
        writer.write_record(&lines);
    }

    writer.flush();
}