complete -c snazy -s r -l regexp -d 'highlight word in a message with a regexp' -r
complete -c snazy -l time-format -d 'Time format' -r
complete -c snazy -s f -l filter-levels -d 'filter levels separated by commas, eg: info,debug' -r -f -a "{info	,debug	,warning	,error	,info	,fatal	,panic	,dpanic	}"
complete -c snazy -s c -l color -d 'When to use colors: never, *auto*, always' -r -f -a "{never	,auto	,always	}"
complete -c snazy -s h -l help -d 'Print help information'
complete -c snazy -s V -l version -d 'Print version information'
complete -c snazy -l kail-no-prefix -d 'Hide container prefix when showing kail'
