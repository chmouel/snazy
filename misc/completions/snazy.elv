
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
            cand -S 'skip line in a message if matching a regexp'
            cand --skip-line-regexp 'skip line in a message if matching a regexp'
            cand -f 'filter by levels'
            cand --filter-level 'filter by levels'
            cand --time-format 'Time format'
            cand --kail-prefix-format 'Kail prefix format'
            cand -k 'key to use for json parsing'
            cand --json-keys 'key to use for json parsing'
            cand --action-regexp 'A regexp to match for action'
            cand --action-command 'An action command to launch when action-regexp match'
            cand -c 'When to use colors: never, *auto*, always'
            cand --color 'When to use colors: never, *auto*, always'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
            cand --kail-no-prefix 'Hide container prefix when showing the log with kail'
            cand -l 'Replace log level with pretty symbols'
            cand --level-symbols 'Replace log level with pretty symbols'
        }
    ]
    $completions[$command]
}
