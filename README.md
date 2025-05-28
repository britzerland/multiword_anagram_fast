# multiword_anagram_fast

A simple fast multi-word anagram solver for python, implemented in rust, with speedups if you know constraints you want to use. Such as maximum number of words, starting letters of words and excluded starting letters.

## install

    pip install multiword_anagram_fast

Or if you are using Google Colab you can: !pip install multiword_anagram_fast 

## usage

    from multiword_anagram_fast import AnagramSolver
    
    solver = AnagramSolver()

    solutions_txt = solve.solve("anagram_this")

>> 11589 solutions saved to ...\anagram_anagram_this_maxW_4_minL_2.txt

    with open(solutions_txt, 'r') as file:
        for line in file:
            print(line.strip())

A more involved example, using constraints:

    phrase = "tendedrosevine"
    must_start_with = "TR"
    must_not_start_with = "DNI"
    max_words = 4
    min_word_length = 2
    timeout_seconds = 30
    max_solutions = 20000

    solver.solve(phrase, must_start_with=must_start_with, must_not_start_with=must_not_start_with,
        max_words=max_words, min_word_length=min_word_length, timeout_seconds=timeout_seconds,
        max_solutions=max_solutions)

All input options and their default settings:

    must_start_with: None
    can_only_ever_start_with: None
    must_not_start_with: None
    max_words: 4
    min_word_length: 2
    timeout_seconds: 30
    max_solutions: 20000
    output_file: "anagram_solutions.txt"

You may wish to change min_word_length to 1 if doing smaller anagrams. With larger anagrams (e.g. 12+ characters) the number of answers begins to explode; so use constraints.


The default dictionary is - [UKACD - around 200k english words allowed in crosswords.](See http://wiki.puzzlers.org/dokuwiki/doku.php?id=solving:wordlists:about:ukacd_readme&rev=1165912949#:~:text=The%20UKACD%20is%20a%20word%20list%20compiled%20for,and%20the%20barred%20puzzles%20in%20the%20Sunday%20broadsheets.). 

You can provide your own word list when loading the solver with:

    solver = AnagramSolver("C://Users/you/files/your_dictionary.txt")

You can also add more words to an already loaded solver, if you know there is additional context that might be included in results that would not be in a standard crossword puzzle answer. 

e.g.
    solver = AnagramSolver() # load standard english dictionary 200k words
    
    # download small wordlist    
    !wget https://raw.githubusercontent.com/britzerland/baronsbafflers/refs/heads/main/blueprince.txt -O blueprince.txt

    # load new words into existing dictionary
    solver.load_dictionary_file("blueprince.txt")

    

# Use default english 

solver.load_dictionary_file("multiword_anagram_fast/dictionaries/ACDLC0A.txt") 
# OR Load your own dictionary text file
# solver.load_dictionary_file(dict_path) 

