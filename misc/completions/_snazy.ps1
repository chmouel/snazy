
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'snazy' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'snazy'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'snazy' {
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'highlight word in a message with a regexp')
            [CompletionResult]::new('--regexp', 'regexp', [CompletionResultType]::ParameterName, 'highlight word in a message with a regexp')
            [CompletionResult]::new('--time-format', 'time-format', [CompletionResultType]::ParameterName, 'Time format')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'filter levels separated by commas, eg: info,debug')
            [CompletionResult]::new('--filter-levels', 'filter-levels', [CompletionResultType]::ParameterName, 'filter levels separated by commas, eg: info,debug')
            [CompletionResult]::new('--kail-no-prefix', 'kail-no-prefix', [CompletionResultType]::ParameterName, 'Hide container prefix when showing kail')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
