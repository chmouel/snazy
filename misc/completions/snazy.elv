
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
            cand --time-format 'Time format'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -f 'filter levels separated by commas, eg: info,debug'
            cand --filter-levels 'filter levels separated by commas, eg: info,debug'
            cand --kail-no-prefix 'Hide container prefix when showing kail'
        }
    ]
    $completions[$command]
}
