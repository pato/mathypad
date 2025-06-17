_mathypad() {
    local cur prev opts
    COMPREPLY=()
    
    # Get current word and previous word
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Available options
    opts="-h -V --help --version --completions"
    
    # Handle --completions argument completion
    if [[ ${prev} == "--completions" ]]; then
        COMPREPLY=( $(compgen -W "bash zsh fish" -- "${cur}") )
        return 0
    fi
    
    # If we're on the first positional argument, provide .pad file completion
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        # Handle options
        if [[ ${cur} == -* ]]; then
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
        fi
        
        # Complete .pad files and directories efficiently
        local -a files=()
        while IFS= read -r -d '' file; do
            if [[ -d "$file" ]]; then
                files+=("$file/")
            elif [[ "$file" == *.pad ]]; then
                files+=("$file")
            fi
        done < <(compgen -f -- "${cur}" | tr '\n' '\0')
        
        COMPREPLY=( "${files[@]}" )
        return 0
    fi
    
    # For other positions, just complete options
    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
    fi
    
    return 0
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _mathypad -o nosort -o bashdefault -o default mathypad
else
    complete -F _mathypad -o bashdefault -o default mathypad
fi

