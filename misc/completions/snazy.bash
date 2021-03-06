_snazy() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            "$1")
                cmd="snazy"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        snazy)
            opts="-h -V -r -S -f -l -k -c --help --version --regexp --skip-line-regexp --filter-level --time-format --kail-prefix-format --kail-no-prefix --level-symbols --json-keys --action-regexp --action-command --color <files>..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --regexp)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --skip-line-regexp)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -S)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --filter-level)
                    COMPREPLY=($(compgen -W "info debug warning error info" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "info debug warning error info" -- "${cur}"))
                    return 0
                    ;;
                --time-format)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --kail-prefix-format)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --json-keys)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -k)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --action-regexp)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --action-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "never auto always" -- "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -W "never auto always" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _snazy -o bashdefault -o default snazy
