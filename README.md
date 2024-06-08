# Codice Fiscale encoder/decoder

## Description

This Rust library provides functionalities to encode and decode Italian fiscal codes. It was developed primarily for personal use, ensuring that the process of working with these codes is straightforward and efficient.

Questa libreria Rust fornisce funzionalità per codificare e decodificare i codici fiscali italiani. È stata sviluppata principalmente per uso personale, garantendo che il processo di lavoro con questi codici sia semplice ed efficiente.

## Features/Caratteristiche

-   Encode an Italian fiscal code from personal information.  
    Codifica di un codice fiscale italiano a partire da informazioni personali.

-   Decode an Italian fiscal code to extract date, gender and place of birth.  
    Decodifica di un codice fiscale italiano per estrarre data, genere e luogo di nascita.

## Installation/Installazione

Add this to your `Cargo.toml`:  
Aggiungi questo al tuo `Cargo.toml`:

```toml
[dependencies]
codice-fiscale = { git = "https://github.com/riccardofano/codice-fiscale", branch = "main" }
```

## Usage/Utilizzo

Here is a simple example of how to use the library:  
Ecco un semplice esempio di come usare la libreria:

```rust
use codice_fiscale::{CodiceFiscale, Gender, NaiveDate, Subject};

// Encoding
let subject = Subject {
    first_name: "Mario".try_into()?,
    last_name: "Rossi".try_into()?,
    gender: Gender::Male,
    birth_date: NaiveDate::from_ymd_opt(1975, 12, 5).unwrap(),
    birth_place: "Roma".try_into()?,
    birth_province: "RM".try_into()?,
};

let encoded_code = CodiceFiscale::encode(&subject)?;
println!("Encoded Fiscale Code: {}", encoded_code.get());

// Decoding
let decoded_info = encoded_code.decode()?;
println!("Decoded Information: {:?}", decoded_info);
```

## License/Licenza

This project is licensed under the MIT License.  
Questo progetto è concesso in licenza sotto la licenza MIT.

I created this library primarily for my own personal use, but feel free to use and modify it as you see fit.  
Ho creato questa libreria principalmente per il mio uso personale, ma sentiti libero di usarla e modificarla come meglio credi.
