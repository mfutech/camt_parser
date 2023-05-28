use csv::{Writer, WriterBuilder};
use minidom::Element;
use minidom::Error as MiniDomError;
use minidom::NSChoice::Any as NSAny;
use select::document::Document;
use select::predicate::{Attr, Name, Text};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Stmt {
    iban: String,
    entries_count: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Ntry {
    account: String,
    date: String,
    description: String,
    debit: String,
    credit: String,
}

fn write_csv_result(
    filename: &str,
    ntry_vec: &Vec<Ntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    // open output csv file
    let file = File::create(filename)?;
    let mut writer = WriterBuilder::new()
        .delimiter(b';')
        .from_writer(BufWriter::new(file)); // and write data in
    for record in ntry_vec {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}
/*se_xml(xml_content: &str) -> Result<Element, MiniDomError> {
    let mut parser = minidom::Parser::new();
    //let dom = parser.parse(xml_content.as_bytes())?;
    Ok(dom.root)
}
*/
fn main() {
    // Open the CAMT.053 file
    let mut file = File::open("file.camt53").expect("Failed to open file");

    // read the file into memory
    let mut xml_content = String::new();
    file.read_to_string(&mut xml_content)
        .expect("Failed to read CAMT53 file");

    // parse XML file
    let xml_content = xml_content.as_str();
    let root_element = xml_content.parse().expect("Failed to parse XML");

    // Extract and process the desired information from the CAMT53 file
    process_camt53(&root_element);
}

fn process_camt53(root_element: &Element) {
    // Parse the XML content
    let customer_statment = root_element.get_child("BkToCstmrStmt", NSAny).unwrap();

    let stmt = customer_statment.get_child("Stmt", NSAny).unwrap();

    // Create a vector to hold the parsed entries
    let mut ntry_vec: Vec<Ntry> = Vec::new();
    // create other data to collect
    let mut stmt_info = Stmt {
        iban: String::from("IBAN"),
        entries_count: 0,
    };

    // iterate over statement children and process according to type
    for child in stmt.children() {
        // data about statment
        if child.is("ElctrncSeqNb", NSAny) {
            stmt_info.entries_count = child.text().parse::<i64>().unwrap();
        }

        // data about account
        if child.is("Acct", NSAny) {
            stmt_info.iban = child
                .get_child("Id", NSAny)
                .and_then(|container| container.get_child("IBAN", NSAny))
                .expect("no IBAN")
                .text();
        }
        // entries
        if child.is("Ntry", NSAny) {
            let res = ntry_parser(stmt_info.iban.clone(), &child);
            ntry_vec.extend(res);
            println!("one record");
        }
    }
    // Print the text content of the first matching <p> element, if any
    println!("First matching element: {:?}", stmt_info);
    write_csv_result("output.csv", &ntry_vec).expect("CSV output failed");
}

fn ntry_parser(account: String, child: &Element) -> Vec<Ntry> {
    let mut result: Vec<Ntry> = Vec::new();
    // let's push some data

    // get amount of entry
    let amount = child
        .get_child("Amt", NSAny)
        .expect("No Amts in Ntry")
        .text();

    // get booking date, which will be used a reference date
    let date = child
        .get_child("BookgDt", NSAny)
        .and_then(|container| container.get_child("Dt", NSAny))
        .expect("no Dt in Bookgdt")
        .text();

    // get NTry description
    let descr = child
        .get_child("AddtlNtryInf", NSAny)
        .expect("cannot get AddtlNtryInf")
        .text();

    // create statement record
    let mut record = Ntry {
        account: account,
        date: date,
        description: descr,
        debit: "0".to_string(),
        credit: "0".to_string(),
    };

    // get type of booking
    let ntry_type = child
        .get_child("CdtDbtInd", NSAny)
        .expect("error in CdtDbtInd")
        .text();

    // push amount in correct field
    // println!("tx type {}", ntry_type);
    if ntry_type.eq("CRDT") {
        record.credit = amount;
    } else {
        record.debit = amount;
    }
    result.push(record);
    return result;
}

fn main_futur() {
    // Open the CAMT.053 file
    let file = File::open("path/to/your/file.camt53").expect("Failed to open file");
    let file = BufReader::new(file);

    // Parse the XML content
    let document = Document::from_read(file).expect("Failed to parse XML document");

    // Example: Extract the text content of specific tags using XPath
    for element in document.find(Name("SomeElement")) {
        let text = element.text();
        println!("Text of SomeElement: {}", text);
    }

    // Example: Extract the attribute value of specific tags using XPath
    for element in document.find(Name("SomeElement[attribute='value']")) {
        let attribute_value = element.attr("attribute");
        println!("Attribute value of SomeElement: {:?}", attribute_value);
    }
}

fn main_old() {
    // Open the CAMT.053 file
    let file = File::open("file.camt53").expect("Failed to open file");
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    let mut writer = Writer::from_path("output.csv").expect("Failed to create CSV writer");

    let mut current_element = String::new();

    // Iterate over the XML events
    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                // Set the current element name
                println!("start {}", name.local_name.clone());
                current_element = name.local_name.clone();
            }
            Ok(XmlEvent::EndElement { name }) => {
                // Process end element based on the current element name
                match current_element.as_str() {
                    "Stmt" => {
                        // Process Stmt end tag
                        println!("End of Stmt");
                        // Write data to CSV file
                        writer
                            .write_record(&["Stmt"])
                            .expect("Failed to write record");
                    }
                    "AnotherElement" => {
                        // Process AnotherElement end tag
                        println!("End of AnotherElement");
                        // Write data to CSV file
                        writer
                            .write_record(&["AnotherElement"])
                            .expect("Failed to write record");
                    }
                    // Handle other elements as needed
                    _ => {}
                }
            }
            Ok(XmlEvent::Characters(text)) => {
                // Process text content based on the current element name
                match current_element.as_str() {
                    "SomeElement" => {
                        // Process text content of SomeElement
                        println!("Text of SomeElement: {}", text);
                        // Write data to CSV file
                        writer
                            .write_record(&[text])
                            .expect("Failed to write record");
                    }
                    "AnotherElement" => {
                        // Process text content of AnotherElement
                        println!("Text of AnotherElement: {}", text);
                        // Write data to CSV file
                        writer
                            .write_record(&[text])
                            .expect("Failed to write record");
                    }
                    // Handle other elements as needed
                    _ => {}
                }
            }
            Err(e) => {
                // Handle XML parsing error
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Flush and close the CSV writer
    writer.flush().expect("Failed to flush CSV writer");
}