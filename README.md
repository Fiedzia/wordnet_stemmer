# rustbar
Wordnet stemmer for rust, based on python nltk implementation.

You will need to download wordnet database files here:
https://wordnet.princeton.edu/wordnet/download/current-version/


```rust
extern crate wordnet_stemmer;

use wordnet_stemmer::{WordnetStemmer, NOUN};

pub fn main(){
    let wn = ::WordnetStemmer::new("/home/maciej/nltk_data/corpora/wordnet/").unwrap();
    println!("{}", wn.lemma(NOUN, "dogs".to_owned());
}

```
