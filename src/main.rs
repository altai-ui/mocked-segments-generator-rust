use rand::RngCore;

use indicatif::{ProgressBar, ProgressStyle};
use requestty::{Answers, Question};

use std::fs::{rename, File};
use std::io::prelude::*;

use fake::faker::internet::raw::FreeEmail;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::uuid::UUIDv4;
use fake::Fake;

use rand::thread_rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn main() -> std::io::Result<()> {
    let quiz_result = call_quiz();

    let mut is_limit_by_count = true;
    let file_limit: u64;

    let result = quiz_result.count;
    match result {
        Some(count) => file_limit = count.try_into().unwrap(),
        None => {
            is_limit_by_count = false;
            file_limit = quiz_result.size.unwrap().try_into().unwrap();
        }
    }

    let mut file_name = quiz_result.name;

    if file_name.is_empty() {
        file_name = "<date>_<type>_<count>.txt".to_string()
    }

    file_name = file_name.replacen("<date>", &chrono::Local::now().to_rfc3339(), 1);
    file_name = file_name.replacen("<type>", get_name_by_type(quiz_result.data_type), 1);

    let mut file = File::create(&file_name)?;

    let pb = ProgressBar::new(file_limit.try_into().unwrap());

    let bar_template = if is_limit_by_count {
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos} / {len} ({eta})"
    } else {
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {decimal_bytes}/{decimal_total_bytes} ({eta})"
    };

    pb.set_style(
        ProgressStyle::default_bar()
            .template(bar_template)
            .progress_chars("#>-"),
    );

    let mut file_line_count: u64 = 0;
    let mut file_size: u64 = 0;

    loop {
        let line = get_line_by_type(quiz_result.data_type);
        let line_bytes = format!("{}\n", line);
        let line_bytes_count: u64 = line_bytes.len().try_into().unwrap();

        file_line_count += 1;
        file_size += line_bytes_count;

        if (is_limit_by_count && file_line_count > file_limit)
            || (!is_limit_by_count && file_size > file_limit)
        {
            file_line_count -= 1;
            break;
        }

        let increment = if is_limit_by_count {
            1
        } else {
            line_bytes_count
        };

        pb.inc(increment);

        file.write_all(format!("{}\n", line).as_bytes())?;
    }

    let new_file_name = file_name.replacen("<count>", &format!("{}", file_line_count), 1);
    rename(file_name, new_file_name)?;

    Ok(())
}

fn is_limit_by_mode_count(answers: &Answers) -> bool {
    answers["limit_by"].as_list_item().unwrap().index == 0
}

fn is_limit_by_mode_size(answers: &Answers) -> bool {
    answers["limit_by"].as_list_item().unwrap().index == 1
}

struct QuizResult {
    name: String,
    data_type: usize,
    count: Option<i64>,
    size: Option<i64>,
}

fn call_quiz() -> QuizResult {
    let questions = vec![
        Question::input("name")
            .message("Название файла")
            .default("<date>_<type>_<count>.txt")
            .build(),
        Question::select("data_type")
            .message("Тип данных?")
            .choices([
                "UUID",
                "MD5",
                "SHA256",
                "Телефонные номера",
                "Электронные почты",
                "MAC-адреса",
            ])
            .build(),
        Question::select("limit_by")
            .message("Ограничение по")
            .choices(["количеству строк", "размеру"])
            .build(),
        Question::int("count")
            .message("Количество строк?")
            .when(is_limit_by_mode_count)
            .build(),
        Question::int("size")
            .message("Максимальный размер в байтах?")
            .when(is_limit_by_mode_size)
            .build(),
    ];

    let answers = requestty::prompt(questions).unwrap();

    QuizResult {
        name: answers["name"].as_string().unwrap().to_string(),
        data_type: answers["data_type"].as_list_item().unwrap().index,
        count: match answers.contains_key("count") {
            true => answers["count"].as_int(),
            false => None,
        },
        size: match answers.contains_key("size") {
            true => answers["size"].as_int(),
            false => None,
        },
    }
}

fn get_name_by_type(key: usize) -> &'static str {
    match key {
        0 => "uuid",
        1 => "md5",
        2 => "sha256",
        3 => "msisdn",
        4 => "email",
        5 => "mac",
        _ => unreachable!(),
    }
}

fn get_line_by_type(key: usize) -> String {
    match key {
        0 => UUIDv4.fake(),
        1 => format!("{:?}", md5::compute(Uuid::new_v4().as_bytes())),
        2 => {
            let mut hasher = Sha256::new();
            hasher.update(Uuid::new_v4().as_bytes());
            format!("{:x}", hasher.finalize())
        }
        3 => NumberWithFormat(EN, "7##########").fake(),
        4 => FreeEmail(EN).fake(),
        5 => {
            let mut octets: [u8; 6] = [0; 6];
            thread_rng().fill_bytes(&mut octets);
            format!(
                // MAC-address
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
            )
        }
        _ => unreachable!(),
    }
}
