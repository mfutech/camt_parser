use csv::WriterBuilder;
use glob::glob;
use minidom::Element;
//use minidom::Error as MiniDomError;
use minidom::NSChoice::Any as NSAny;
use std::fs::File;
use std::io::{BufWriter, Read};

// cli
use clap::{Arg, Command};

// Statement
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Stmt {
    iban: String,
    entries_count: i64,
}

// Entry (NTry)
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct Ntry {
    account: String,     // Account
    date: String,        // date
    description: String, //description of transaction
    debit: String,       // debit amount
    credit: String,      // credit amount
    ntry_type: String,   // type of entry
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

fn main() {
    let matches = Command::new("CAMT53 parser")
        .author("mfutech")
        .version("1.0.0")
        .about("export all transaction of a CAMT53 into a csv file")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output file to use")
                .default_value("output.csv"),
        )
        .arg(
            Arg::new("input_files")
                .trailing_var_arg(true)
                .num_args(1..=100)
                .value_name("FILE")
                .help("file to be parsed CAMT53 format")
                .default_value("*.xml"),
        )
        /*        .after_help(
                    "Longer explanation to appear after the options when \
                         displaying the help information from --help or -h",
                )
        */
        .get_matches();

    let output_filename = matches
        .get_one::<String>("output")
        .expect("need an output file");

    let input_filenames = matches
        .get_many::<String>("input_files")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    let mut entries = Vec::<Ntry>::new();

    for filenames in input_filenames {
        for filename in glob(filenames).expect("invalid glob pattern") {
            // Open the CAMT.053 file
            let filename = filename.unwrap();
            let mut file = File::open(filename.clone()).expect("Failed to open file");

            println!("processing file: {:?}", filename);

            // read the file into memory
            let mut xml_content = String::new();
            file.read_to_string(&mut xml_content)
                .expect("Failed to read CAMT53 file");

            // parse XML file
            let xml_content = xml_content.as_str();
            let root_element = xml_content.parse().expect("Failed to parse XML");

            // Extract and process the desired information from the CAMT53 file
            let result = process_camt53(&root_element);
            entries.extend(result);
        }
    }

    write_csv_result(output_filename, &entries).expect("CSV output failed");
}

fn process_camt53(root_element: &Element) -> Vec<Ntry> {
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
            // DEBUG // println!("one record");
        }
    }
    return ntry_vec;
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

    // get type of booking
    let ntry_type = child
        .get_child("CdtDbtInd", NSAny)
        .expect("error in CdtDbtInd")
        .text();

    // create statement record
    let mut record = Ntry {
        account: account,
        date: date,
        description: descr,
        debit: "0".to_string(),
        credit: "0".to_string(),
        ntry_type: ntry_type,
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

    let mut had_ntry_dtls = false;
    for entry in child.children() {
        if entry.is("NtryDtls", NSAny) {
            // DEBUG // println!("found NtryDtls");
            for ntry_dtls_child in entry.children() {
                if ntry_dtls_child.is("TxDtls", NSAny) {
                    // DEBUG // println!("found txdtls");
                    let txdtls = txdtls_parser(&record, ntry_dtls_child);
                    result.push(txdtls);
                    had_ntry_dtls = true;
                }
            }
        }
    }

    if had_ntry_dtls == false {
        result.push(record)
    }
    return result;
}

fn txdtls_parser(entry: &Ntry, tx_dtls: &Element) -> Ntry {
    // DEBUG // println!("found a txdtls");
    let mut result = entry.clone();
    let mut operation = Err(());
    let mut amount = Err(());

    for child in tx_dtls.children() {
        // amount of transaction
        if child.is("Amt", NSAny) {
            amount = Ok(child.text());
        }

        // type of transaction
        if child.is("CdtDbtInd", NSAny) {
            operation = Ok(child.text());
        }

        // corresponding party
        if child.is("RltdPties", NSAny) {
            // find either Cdtr or Dbtr Nm
            let mut partner_nm = "unknown_partner".to_string();
            let mut iban = "unknown_iban".to_string();
            let not_found_element = Element::builder("NotFound", "NotFound")
                .append("Not Found")
                .build();

            if let Some(cdtr) = child.get_child("Cdtr", NSAny) {
                partner_nm = cdtr.get_child("Nm", NSAny).expect("Cdtr without Nm").text();
                match child.get_child("CdtrAcct", NSAny) {
                    Some(cdtracct) => {
                        iban = cdtracct
                            .get_child("Id", NSAny)
                            .and_then(|container| container.get_child("IBAN", NSAny))
                            .expect("no cdtr IBAN in RltdPties")
                            .text();
                    }
                    _ => iban = "no IBAN".to_string(),
                }
            }

            if let Some(dbtr) = child.get_child("Dbtr", NSAny) {
                partner_nm = dbtr.get_child("Nm", NSAny).expect("Cdtr without Nm").text();
                match child.get_child("DbtrAcct", NSAny) {
                    Some(dbtracct) => {
                        iban = dbtracct
                            .get_child("Id", NSAny)
                            .and_then(|container| container.get_child("IBAN", NSAny))
                            .unwrap_or(&not_found_element)
                            .text()
                    }

                    _ => iban = "UKNOWN IBAN".to_string(),
                }
            }

            let mut description = partner_nm;
            description.push_str(" - ");
            description.push_str(&iban);
            result.description = description;
        }

        // Remote Information / Ustrd
        if child.is("RmtInf", NSAny) {
            let ustrd = child
                .get_child("Ustrd", NSAny)
                .expect("RmtInf without Ustrd")
                .text();
            result.description.push_str(&ustrd);
        }
    }
    let amount = amount.expect("did not find amount");
    if operation.expect("did not found operation type").eq("DBIT") {
        result.debit = amount;
        result.credit = "0".to_string();
    } else {
        result.credit = amount;
        result.debit = "0".to_string();
    }
    // DEBUG // println!("found {:?}", result);
    return result;
}
