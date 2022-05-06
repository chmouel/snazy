complete -c snazy -s r -l regexp -d 'highlight word in a message with a regexp' -r
complete -c snazy -s f -l filter-level -d 'filter by levels' -r -f -a "{info	,debug	,warning	,error	,info	}"
complete -c snazy -l time-format -d 'Time format' -r
complete -c snazy -l kail-prefix-format -d 'Kail prefix format' -r
complete -c snazy -s k -l json-keys -d 'key to use for json parsing' -r
complete -c snazy -l action-regexp -d 'A regexp to match for action' -r
complete -c snazy -l action-command -d 'An action command to launch when action-regexp match' -r
complete -c snazy -s c -l color -d 'When to use colors: never, *auto*, always' -r -f -a "{never	,auto	,always	}"
complete -c snazy -s h -l help -d 'Print help information'
complete -c snazy -s V -l version -d 'Print version information'
complete -c snazy -l kail-no-prefix -d 'Hide container prefix when showing the log with kail'
complete -c snazy -s l -l level-symbols -d 'Replace log level with pretty symbols'
