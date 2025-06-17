_mathypad() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="mathypad"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        mathypad)
            opts="-h -V --help --version --completions"
            
            # If we're on the first positional argument, provide .pad file completion
            if [[ ${COMP_CWORD} -eq 1 ]] ; then
                # Handle options
                if [[ ${cur} == -* ]] ; then
                    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                    return 0
                fi
                
                # Complete .pad files
                local files
                files=$(compgen -f -- "${cur}")
                if [[ -n "$files" ]]; then
                    # Filter for .pad files and directories
                    COMPREPLY=()
                    while IFS= read -r file; do
                        if [[ -d "$file" ]]; then
                            COMPREPLY+=("$file/")
                        elif [[ "$file" == *.pad ]]; then
                            COMPREPLY+=("$file")
                        fi
                    done <<< "$files"
                fi
                return 0
            fi
            
            # Handle --completions argument completion
            if [[ ${prev} == "--completions" ]] ; then
                COMPREPLY=( $(compgen -W "bash zsh fish" -- "${cur}") )
                return 0
            fi
            
            # For other positions, just complete options
            if [[ ${cur} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _mathypad -o nosort -o bashdefault -o default mathypad
else
    complete -F _mathypad -o bashdefault -o default mathypad
fi
