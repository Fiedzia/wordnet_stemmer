use std::collections::{HashMap, HashSet, hash_map};
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind, Result};
use std::vec::Vec;


#[derive(PartialEq)]
pub enum Part {
    Noun=0,
    Verb=1,
    Adj=2,
    Adv=3
}

impl Part {

    fn as_usize(&self) -> usize {
        match *self {
            Part::Noun => 0,
            Part::Verb => 1,
            Part::Adj => 2,
            Part::Adv => 3
        }
    }

    fn as_str(&self) -> &str {
        match *self {
            Part::Noun => "noun",
            Part::Verb => "verb",
            Part::Adj => "adj",
            Part::Adv => "adv"
        }
    } 

}

    
pub const NOUN:usize = 0;
pub const VERB:usize = 1;
pub const ADJ:usize = 2;
pub const ADV:usize = 3;

const PARTS:[usize; 4] = [NOUN, VERB, ADJ, ADV];
const WN_FILES:[[&'static str; 2]; 4] = [
    /* noun */ ["index.noun", "noun.exc"],
    /* verb */ ["index.verb", "verb.exc"],
    /* adj  */ ["index.adj", "adj.exc"],
    /* adv  */ ["index.adv", "adv.exc"]
];


/*static STR_ADJ: char = 'a';
static STR_ADJ_SAT: char = 's';
static STR_ADV: char = 'r';
static STR_NOUN: char = 'n';
static STR_VERB: char = 'v';*/

type FastHashMap = HashMap<String, String>;
type Wordlist = Vec<FastHashMap>;
type Exceptions = Vec<HashMap<String, Vec<String>>>;
type Substitutions = Vec<Vec<Vec<&'static str>>> ;
type LemmaPosOffsetMap = HashMap<String, HashMap<usize, Vec<i32>>>;
//type FileMap = HashMap<char, String>;

#[derive(Clone,Debug)]
pub struct WordnetStemmer {
    wordlist: Wordlist,
    exceptions: Exceptions,
    substitutions: Substitutions,
    lemma_pos_offset_map: LemmaPosOffsetMap,
    basedir: String,
}

impl WordnetStemmer {

    pub fn new(basedir: &str) -> Result<WordnetStemmer> {
        let mut wn = WordnetStemmer {
            basedir: basedir.to_owned(),
            wordlist: Vec::new(),
            exceptions: Vec::new(),
            substitutions: vec![ 
                //noun 
                vec![
                  vec!["s",    ""   ],
                  vec!["ses",  "s"  ],
                  vec!["ves",  "f"  ],
                  vec!["xes",  "x"  ],
                  vec!["zes",  "z"  ],
                  vec!["ches", "ch" ],
                  vec!["shes", "sh" ],
                  vec!["men",  "man"],
                  vec!["ies",  "y"  ]
                ],
                //verb 
                vec![
                  vec!["s",   "" ],
                  vec!["ies", "y"],
                  vec!["es",  "e"],
                  vec!["es",  "" ],
                  vec!["ed",  "e"],
                  vec!["ed",  "" ],
                  vec!["ing", "e"],
                  vec!["ing", "" ]
                ],
                //adj 
                vec![
                  vec!["er",  "" ],
                  vec!["est", "" ],
                  vec!["er",  "e"],
                  vec!["est", "e"]
                ],
                //adv 
                vec![],
            ],
            lemma_pos_offset_map: HashMap::new(),
        };

        for _ in PARTS.iter() {
            wn.wordlist.push(Default::default());
            wn.exceptions.push(Default::default());
        }
        for part in PARTS.iter() {
            wn.load(*part, WN_FILES[*part])?;
        }

      
        Ok(wn)
    }

    /*fn filemap() -> FileMap {
        let mut fm = HashMap::new();
        fm.insert(STR_ADJ, "adj".to_owned());
        fm.insert(STR_ADV, "adv".to_owned());
        fm.insert(STR_NOUN, "noun".to_owned());
        fm.insert(STR_VERB, "verb".to_owned());

        fm
    }*/

    fn load(&mut self, 
        part: usize,
        pair: [&str; 2]
        ) -> Result<()> {
        let fname:String = format!("{}{}", self.basedir, pair[0]);
        let mut f = match File::open(fname.clone()) {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => return Err(io::Error::new(ErrorKind::Other, format!("WordnetStemmer: could not open or read file {}", fname))),
                _ => return Err(e)
            }
        };
        let mut br = BufReader::new(f);
        for line_result in br.lines() {
            let line = line_result?;
            if line.starts_with("  ") {
                continue
            }
    
            let word = line.splitn(2, ' ').nth(0).unwrap();
            self.wordlist[part].insert(word.clone().to_owned(), word.to_owned());
        }
    
    
        let fname = format!("{}{}", self.basedir, pair[1]);
        f = File::open(fname)?;
        br = BufReader::new(f);
        for line_result in br.lines() {
            let line: String = line_result?;
            if line.starts_with("  ") {
                continue
            }
            
            let words: Vec<&str> = line.splitn(3, ' ').collect();
            self.exceptions[part].entry(words[0].to_owned()).or_insert(Vec::new()).push(words[1].to_owned());
        }
         
        let res = self.load_lemma_pos_offset_map();
        res
    }

    fn load_lemma_pos_offset_map(&mut self) -> Result<()>{
       for variant in [Part::Noun, Part::Verb, Part::Adj, Part::Adv].iter() {
       //for suffix in WordnetStemmer::filemap().values(){
            let fname:String = format!("{}/index.{}", self.basedir, variant.as_str());
            let f = match File::open(fname.clone()) {
                Ok(v) => v,
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => return Err(io::Error::new(ErrorKind::Other, format!("WordnetStemmer: could not open or read file {}", fname))),
                    _ => return Err(e)
                }
            };
            let br = BufReader::new(f);
            for line_result in br.lines() {
                let line = line_result?;
                if line.starts_with(" ") {
                    continue
                }
                let mut iter = line.split(' ');

                // get the lemma and part-of-speech
                let lemma = iter.next().unwrap();
                let _ = iter.next().unwrap(); //pos
                let n_synsets = iter.next().unwrap().parse::<i32>().unwrap();
                // assert!(n_synsets > 0)
                
                let n_pointers = iter.next().unwrap().parse::<i32>().unwrap();

                // same as number of synsets
                let _ = iter.nth(n_pointers as usize).unwrap().parse::<i32>().unwrap(); //n_senses

                // get number of senses ranked according to frequency
                let _ =iter.next();
                // get synset offsets
                let synset_offsets:Vec<i32> = iter.take(n_synsets as usize).map(|x|x.parse::<i32>().unwrap()).collect();
                match self.lemma_pos_offset_map.entry(lemma.to_owned()) {
                    hash_map::Entry::Vacant(entry) => {
                        let mut hm = HashMap::new();
                        hm.insert(variant.as_usize(), synset_offsets);
                        entry.insert(hm);
                    },
                    hash_map::Entry::Occupied(mut entry) => { entry.get_mut().insert(variant.as_usize(), synset_offsets); }
                }
                //let mut hm = HashMap::new();
                //hm.insert(variant.as_usize(), synset_offsets);
                //self.lemma_pos_offset_map.insert(lemma.to_owned(), hm);
                if *variant == Part::Adj {
                }


            }
           
       }
       Ok(())
    } 

    fn apply_rules(&self, part: usize, words: &Vec<String>
    ) -> Vec<String>{
        let mut result = vec![];
        for word in words.iter() {
            for pair in self.substitutions[part].iter(){
                let old: &str = (*pair)[0];
                let new: &str = (*pair)[1];
                if word.ends_with(old){
                    let w: String = (word.chars().take(word.chars().count() - old.chars().count()).collect::<String>()) + &new;
                    result.push(w);

                }
            }

        }
        result
    }

    fn filter_forms(&self, words: &Vec<String>, part: usize) -> Vec<String> {
        let mut result:Vec<String> = vec![];
        let mut seen = HashSet::new();
        for word in words.iter(){
            if self.lemma_pos_offset_map.contains_key(word) {
                if self.lemma_pos_offset_map[word].contains_key(&part) {
                    if !seen.contains(word) {
                        seen.insert(word);
                        result.push(word.to_owned());
                    }
                }
            }
        }
        result
    }

    fn morphy(&self,
        part: usize,
        word: &str
    ) -> Vec<String> {
    
        if self.exceptions[part].contains_key(word){
            let mut words = vec![word.to_owned()];
            words.extend_from_slice(&self.exceptions[part][word]);
            return self.filter_forms(&words, part)
        }
    
        let mut forms = self.apply_rules(part, &vec![word.to_owned()]);
        {
            let mut words = vec![word.to_owned()];
            words.extend_from_slice(&forms);
            let results = self.filter_forms(&words, part);
            if results.len() > 0 {
                return results
            }
        }
        
        while forms.len() > 0 {
            forms = self.apply_rules(part, &forms);
            let results = self.filter_forms(&forms, part);
            if results.len() > 0 {
                return results
            }
        }
        vec![]
    }
    
    pub fn lemma(&self, part: usize, word: &str) -> String {
        let lemmas = self.morphy(part, &word);
        if lemmas.len() > 0 {
            let mut w_idx = 0;
            let mut w_min_len = lemmas[0].len();
            for pair in lemmas.iter().enumerate() {
                let (idx, w2) = pair;
                if w2.len() < w_min_len {
                    w_min_len = w2.len();
                    w_idx = idx
                }
            }
            lemmas[w_idx].to_owned()
        } else { word.to_owned() }
    }

    pub fn lemma_phrase(&self, part: usize, phrase: &str) -> String {
        phrase
        .to_lowercase()
        .split_whitespace()
        .map(|word| self.lemma(part, word))
        .collect::<Vec<String>>()
        .join(" ")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_stemming() {
        let wn = ::WordnetStemmer::new("/home/maciej/nltk_data/corpora/wordnet/").unwrap();
        for (word, expected) in vec![
            ("dogs", "dog"),
            ("money", "money"),
            ("bananas", "banana"),
            ("berries", "berry"),
            ("press", "press"),
            ("ferries", "ferry"),
        ] {
            assert_eq!(expected.to_owned(), wn.lemma(::NOUN, word.to_owned()) );
        } 
    }
}
