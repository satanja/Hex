cargo build --release --bin dfvstritus
Get-ChildItem "./instances" | 
Foreach-Object {
    $content = Get-Content $_.FullName
    $name = "k" + $_.Name
    Measure-Command {
        $content | ./target/release/dfvstritus.exe | Set-Content -Path "./solutions/$name"
    } | Select-Object -Property TotalSeconds
}