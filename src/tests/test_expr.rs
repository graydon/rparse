// Test a grammar capable of evaluating simple mathematical expressions.
import parsers::*;
import primitives::*;
import test_helpers::*;
import types::*;

fn expr_parser() -> parser<int>
{
	// Create parsers for punctuation and integer literals. All of these
	// parsers allow for zero or more trailing whitespace characters.
	let plus_sign = text("+").space0();
	let minus_sign = text("-").space0();
	let mult_sign = text("*").space0();
	let div_sign = text("/").space0();
	let left_paren = text("(").space0();
	let right_paren = text(")").space0();
	let int_literal = integer().space0();
	
	// Parenthesized expressions require a forward reference to the expr parser
	// so we initialize a function pointer to something of the right type, create
	// a parser using the parser expr_ptr points to, and fixup expr_ptr later.
	let expr_ptr = @mut return(0);
	let expr_ref = forward_ref(expr_ptr);
	
	// sub_expr := [-+]? '(' expr ')'
	let sub_expr = alternative([
		sequence4(plus_sign, left_paren, expr_ref, right_paren) {|_a, _b, c, _d| c},
		sequence4(minus_sign, left_paren, expr_ref, right_paren) {|_a, _b, c, _d| -c},
		sequence3(left_paren, expr_ref, right_paren) {|_a, b, _c| b}]);
	
	// factor := integer | sub_expr
	// The tag provides better error messages if the factor parser fails
	// on the very first character.
	let factor = int_literal.or(sub_expr).tag("integer or sub-expression");
	
	// term := factor ([*/] factor)*
	let term = factor.chainl1(mult_sign.or(div_sign))
		{|lhs, op, rhs| if op == "*" {lhs*rhs} else {lhs/rhs}};
	
	// expr := term ([+-] term)*
	let expr = term.chainl1(plus_sign.or(minus_sign))
		{|lhs, op, rhs| if op == "+" {lhs + rhs} else {lhs - rhs}};
	*expr_ptr = expr;
	
	// start := space0 expr EOT
	// The s syntax is a little goofy because the space0 comes before 
	// instead of after expr so it needs to be told which type to use.
	let s = return(0).space0();
	let start = everything(expr, s);
	
	ret start;
}

#[test]
fn test_factor()
{
	let p = expr_parser();
	
	assert check_int_failed("", p, "integer or sub-expression", 1);
	assert check_int_ok("23", p, 23);
	assert check_int_ok(" 57   ", p, 57);
	assert check_int_ok("\t\t\n-100", p, -100);
	assert check_int_ok("+1", p, 1);
	assert check_int_failed("+", p, "digits or '('", 1);
	assert check_int_failed(" 57   200", p, "EOT", 1);
	
	assert check_int_ok("(23)", p, 23);
	assert check_int_ok("((23))", p, 23);
	assert check_int_failed("(23", p, "')'", 1);
	assert check_int_failed("((23)", p, "')'", 1);
	
	assert check_int_ok("-(23)", p, -23);
	assert check_int_ok("+(5)", p, 5);
}

#[test]
fn test_term()
{
	let p = expr_parser();
	
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok(" 4 / 2   ", p, 2);
	assert check_int_failed("4 * ", p, "EOT", 1);
	assert check_int_failed("4 ** 1", p, "EOT", 1);
	assert check_int_failed("4 % 1", p, "EOT", 1);
	
	assert check_int_ok("2 * 3 / 6", p, 1);
}

#[test]
fn test_expr()
{
	let p = expr_parser();
	
	assert check_int_ok("3+2", p, 5);
	assert check_int_ok(" 3\t-2  ", p, 1);
	assert check_int_ok("2 + 3*4", p, 14);
	assert check_int_ok("(2 + 3)*4", p, 20);
}
