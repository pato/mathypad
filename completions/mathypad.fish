# Fish completion for mathypad

complete -c mathypad -s h -l help -d 'Print help information'
complete -c mathypad -s V -l version -d 'Print version information'
complete -c mathypad -l completions -d 'Generate shell completion files' -x -a 'bash zsh fish'

# Complete .pad files for the first argument
complete -c mathypad -n '__fish_is_first_arg' -f -a '(__fish_complete_suffix .pad)'

