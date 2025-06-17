# Ensure completion system is loaded
if ! (( $+functions[compdef] )); then
    autoload -U compinit && compinit
fi

_mathypad() {
    local -a opts
    opts=(
        '(-h --help)'{-h,--help}'[Print help information]'
        '(-V --version)'{-V,--version}'[Print version information]'
        '--completions[Generate shell completion files]:shell:(bash zsh fish)'
        '1: :_mathypad_pad_files'
        '*:: :_files'
    )
    
    _arguments -C $opts
}

# Custom completion function for .pad files
_mathypad_pad_files() {
    _files -g '*.pad'
}

# Register the completion
compdef _mathypad mathypad

