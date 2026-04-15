grep -n "upload_device_ms" *.md | perl -ne '
    my ($file_linenum) = /^(.+): /;
    my @vals;
    while (/=(\d+\.?\d*)(m?s)/g) {
        push @vals, $2 eq "ms" ? $1 : $1 * 1000;
    }
    print "$file_linenum: |" . join("|", @vals) . "|\n";
'

grep "get_device_ms" *.md | perl -ne '
    my @vals;
    while (/=(\d+\.?\d*)(m?s)/g) {
        push @vals, $2 eq "ms" ? $1 : $1 * 1000;
    }
    print "|" . join("|", @vals) . "|\n";
'

grep "get_device_history_ms" *.md | perl -ne '
    my @vals;
    while (/=(\d+\.?\d*)(m?s)/g) {
        push @vals, $2 eq "ms" ? $1 : $1 * 1000;
    }
    print "|" . join("|", @vals) . "|\n";
'
