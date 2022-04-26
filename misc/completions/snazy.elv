
use builtin;
use str;

set edit:completion:arg-completer[snazy] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'snazy'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'snazy'= {
            cand -r 'highlight word in a message with a regexp'
            cand --regexp 'highlight word in a message with a regexp'
            cand -f 'filter by levels'
            cand --filter-levels 'filter by levels'
            cand --time-format 'Time format'
            cand -k 'key to use for json parsing'
            cand --json-keys 'key to use for json parsing'
            cand -c 'When to use colors: never, *auto*, always'
            cand --color 'When to use colors: never, *auto*, always'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
            cand --kail-no-prefix 'Hide container prefix when showing the log with kail'
        }
    ]
    $completions[$command]
}
