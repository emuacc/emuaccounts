#!/usr/bin/env perl
use strict;
use warnings;

my ($path) = @ARGV;
die "usage: $0 <generated-parser.rs>\n" unless defined $path;

open my $input, '<', $path or die "cannot read $path: $!\n";
local $/;
my $source = <$input>;
close $input;

# antlr4rust v0.5.0's Rust target does not complete its -no-listener mode:
# BaseParser still requires each context to implement the generic Listenable
# contract. Supply the no-op generic implementation without generating a
# grammar-specific listener API or listener source files.
if ($source =~ /pub trait ([A-Za-z0-9_]+Context)<'input>:/) {
    my $context_trait = $1;
    (my $context_type = $context_trait) =~ s/Context$/ContextType/;
    (my $grammar_name = $context_trait) =~ s/Context$//;
    my $listener_trait = "${grammar_name}NoOpListener";

    my $listener_compat = <<"RUST";
// antlr4rust runtime compatibility for visitor-only generation.
pub trait $listener_trait<'input>: ParseTreeListener<'input,$context_type> {}
antlr4rust::coerce_from!{ 'input : $listener_trait<'input> }
antlr4rust::tid! { impl<'input> TidAble<'input> for dyn $listener_trait<'input> + 'input }

RUST
    $source =~ s/(type BaseParserType<'input, I> =\n)/$listener_compat$1/
        or die "cannot insert listener compatibility trait in $path\n";

    $source =~ s{
        (BaseParser<'input,[^\n]+?,\s*I,\s*\Q$context_type\E)\s*>
    }{$1, dyn $listener_trait<'input> + 'input >}x
        or die "cannot patch BaseParser type in $path\n";

    $source =~ s{
        (ParseTreeWalker<'input,\s*'a,\s*\Q$context_type\E)\s*>
    }{$1, dyn $listener_trait<'input> + 'a >}x
        or die "cannot patch ParseTreeWalker type in $path\n";

    my $trait_header = qr/
        (pub\ trait\ \Q$context_trait\E<'input>:\n
        \s*for<'x>\ Visitable<[^\n]+\+\s*\n)
    /x;
    my $listener_bound =
        "\tfor<'x> Listenable<dyn $listener_trait<'input> + 'x > + \n";
    $source =~ s/$trait_header/$1$listener_bound/
        or die "cannot patch listener bound in $path\n";

    my $context_impl = qr{
        (impl<'input>\s+\Q$context_trait\E<'input>\s+for\s+
        ([A-Za-z0-9_]+Context)<'input>\{\}\n)
    }x;
    $source =~ s{$context_impl}{
        my ($original, $concrete) = ($1, $2);
        $original
            . "\nimpl<'input,'a> Listenable<dyn $listener_trait<'input> + 'a> "
            . "for $concrete<'input>{}\n";
    }eg;

    $source =~ s/^.*TidAble.*Listener.*\n//mg;
    # The broad cleanup above also sees NoOpListener; restore its local Tid.
    $source =~ s{
        (antlr4rust::coerce_from!\{\ 'input\ :\ \Q$listener_trait\E<'input>\ \}\n)
    }{$1antlr4rust::tid! { impl<'input> TidAble<'input> for dyn $listener_trait<'input> + 'input }\n}x;
    $source =~ s/(#!\[allow\(unused_braces\)\]\n)/$1#![allow(unused_parens)]\n/;
}

open my $output, '>', $path or die "cannot write $path: $!\n";
print {$output} $source;
close $output;
