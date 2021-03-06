extern crate reqwest; // 0.9.18
extern crate zip;
extern crate rustlearn;
extern crate time;

use rustlearn::prelude::*;
use std::io::{Cursor, Read};
use zip::read::ZipArchive;
use rustlearn::feature_extraction::DictVectorizer;
use rustlearn::cross_validation::CrossValidation;
use rustlearn::linear_models::sgdclassifier;
use rustlearn::metrics::accuracy_score;

fn main() -> Result<(), String> {
    let zipped = download(
        "https://archive.ics.uci.edu/ml/machine-learning-databases/00228/smsspamcollection.zip",
    );
    let raw_data = unzip(zipped);

    let (x,y) = parse(&raw_data);
    println!("X: {} rows, {} columns, {} non-zero entries",
          x.rows(), x.cols(), x.nnz());
    let start_time = time::precise_time_ns();
let (test_accuracy, train_accuracy) = fit(&x, &y);
println!("Test accuracy: {:.3}, train accuracy: {:.3}",
         test_accuracy, train_accuracy);
let duration = time::precise_time_ns() - start_time;

println!("Training time: {:.3} seconds",
         duration as f64 / 1.0e+9);

    Ok(())
}

fn download(url: &str) -> Vec<u8> {
    let mut body = reqwest::get(url).expect("error");

    let mut data = Vec::new();
    body.read_to_end(&mut data).unwrap();

    data
}
fn unzip(zipped: Vec<u8>) -> String {
    let mut archive = ZipArchive::new(Cursor::new(zipped)).unwrap();
    let mut file = archive.by_name("SMSSpamCollection").unwrap();

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    data
}
fn parse(data: &str) -> (SparseRowArray, Array) {

    // Initialise the vectorizer
    let mut vectorizer = DictVectorizer::new();
    let mut labels = Vec::new();

    // Like Python enumerate(), this will iterate
    // over pairs of (row_number, line).
    for (row_num, line) in data.lines().enumerate() {
        // The label and text message content is separated by a tab.
        // We split the line in two here.
        let (label, text) = line.split_at(line.find('\t').unwrap());

        // Convert the labels to binary. We use pattern matching
        // to ensure that the program is aborted if an unexpected
        // label is encountered.
        labels.push(match label {
            "spam" => 0.0,
            "ham" => 1.0,
            _ => panic!(format!("Invalid label: {}", label))
        });

        // The vectorizer will keep a mapping from tokens
        // to column indices.
        for token in text.split_whitespace() {
            vectorizer.partial_fit(row_num, token, 1.0);
        }
    }

    (vectorizer.transform(), Array::from(labels))
}

fn fit(x: &SparseRowArray, y:&Array) -> (f32, f32) {
    

    let num_epochs = 10;
    let num_folds = 10;

    let mut test_accuracy = 0.0;
    let mut train_accuracy = 0.0;

    // The cross validation interator returns indices of train and test rows
    for (train_indices, test_indices) in CrossValidation::new(y.rows(), num_folds) {

        // Slice the feature matrices
        let X_train = x.get_rows(&train_indices);
        let X_test = x.get_rows(&test_indices);

        // Slice the target vectors
        let y_train = y.get_rows(&train_indices);
        let y_test = y.get_rows(&test_indices);

        let mut model = sgdclassifier::Hyperparameters::new(x.cols())
            .learning_rate(0.05)
            .l2_penalty(0.01)
            .build();

        // Repeated calls to `fit` perform epochs of training
        for _ in 0..num_epochs {
            model.fit(&X_train, &y_train).unwrap();
        }

        let fold_test_accuracy = accuracy_score(&y_test, &model.predict(&X_test).unwrap());
        let fold_train_accuracy = accuracy_score(&y_train, &model.predict(&X_train).unwrap());

        test_accuracy += fold_test_accuracy;
        train_accuracy += fold_train_accuracy;
    }

    (test_accuracy / num_folds as f32,
     train_accuracy / num_folds as f32)
}
