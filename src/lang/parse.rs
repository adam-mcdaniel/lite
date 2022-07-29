use super::{Expr, Env, err};
use std::collections::BTreeMap;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "lang/grammar.pest"]
pub struct LangParser;

pub fn parse(code: &str) -> Result<Expr, Expr> {
    use pest::Parser;
    match LangParser::parse(Rule::program, code) {
        Ok(pairs) => {
            let mut exprs = vec![];
            for pair in pairs {
                match process_expr(pair) {
                    Ok(e) => exprs.push(e),
                    Err(e) => return err("InvalidSyntax", Expr::String(code.to_owned())),
                }
            }
            Ok(Expr::Do(exprs))
        },
        Err(e) => err("InvalidSyntax", Expr::String(code.to_owned())),
    }
}


fn process_expr(pair: Pair<Rule>) -> Result<Expr, String> {
    Ok(match pair.as_rule() {
        Rule::value | Rule::complex_value | Rule::simple_value | Rule::program => {
            process_expr(pair.into_inner().next().unwrap())?
        }
        Rule::number => {
            if let Ok(n) = pair.as_str().parse::<isize>() {
                Expr::Int(n)
            } else {
                Expr::Float(pair.as_str().parse::<f64>().unwrap().into())
            }
        }
        Rule::string => Expr::String(
            pair.into_inner()
                .next()
                .unwrap()
                .as_str()
                .replace("\\n", "\n")
                .replace("\\\"", "\"")
        ),
        Rule::symbol => Expr::Symbol(pair.as_str().to_string()),

        Rule::suite => {
            let mut list = Vec::new();
            for pair in pair.into_inner() {
                list.push(process_expr(pair)?);
            }
            if list.len() == 1 {
                list[0].clone()
            } else {
                Expr::Do(list)
            }
        }

        Rule::list => {
            let mut list = Vec::new();
            for pair in pair.into_inner() {
                list.push(process_expr(pair)?);
            }
            Expr::List(list)
        }
        Rule::dict => {
            let mut dict = BTreeMap::new();
            for pair in pair.into_inner() {
                let mut pairs = pair.into_inner();
                let key = process_expr(pairs.next().unwrap())?;
                let value = process_expr(pairs.next().unwrap())?;
                dict.insert(key, value);
            }
            Expr::Dict(dict)
        }
        Rule::group => Expr::Group(Box::new(process_expr(pair.into_inner().next().unwrap())?)),
        Rule::boolean => {
            if pair.as_str() == "true" {
                Expr::Bool(true)
            } else {
                Expr::Bool(false)
            }
        }

        Rule::quote => Expr::Quote(Box::new(process_expr(pair.into_inner().next().unwrap())?)),

        Rule::let_var_in => {
            let mut pairs = pair.into_inner();
            let var = process_expr(pairs.next().unwrap())?;
            let val = process_expr(pairs.next().unwrap())?;
            let ret = process_expr(pairs.next().unwrap())?;
            Expr::Let(Box::new(var), Box::new(val), Box::new(ret))
        }

        Rule::set_var => {
            let mut pairs = pair.into_inner();
            let var = process_expr(pairs.next().unwrap())?;
            let val = process_expr(pairs.next().unwrap())?;
            Expr::Assign(Box::new(var), Box::new(val))
        }

        Rule::if_else => {
            let mut pairs = pair.into_inner();
            let cond = process_expr(pairs.next().unwrap())?;
            let then = process_expr(pairs.next().unwrap())?;
            let else_ = process_expr(pairs.next().unwrap())?;
            Expr::If(Box::new(cond), Box::new(then), Box::new(else_))
        }

        Rule::for_loop => {
            let mut pairs = pair.into_inner();
            let name = process_expr(pairs.next().unwrap())?;
            let list = process_expr(pairs.next().unwrap())?;
            let e = process_expr(pairs.next().unwrap())?;
            Expr::For(Box::new(name), Box::new(list), Box::new(e))
        }

        Rule::apply => {
            let mut pairs = pair.into_inner();
            // println!("{:?}", pairs);
            let mut f = process_expr(pairs.next().unwrap())?;
            let mut args = vec![];
            for pair in pairs {
                args.push(process_expr(pair)?)
            }
            Expr::Apply(Box::new(f), args)
        }

        Rule::lambda_fn => {
            let mut pairs = pair.into_inner();
            let arg = process_expr(pairs.next().unwrap())?;
            let ret = process_expr(pairs.next().unwrap())?;
            Expr::Fn(vec![arg], Box::new(ret), Env::default())
        }

        Rule::proc_fn => {
            let mut pairs = pair.into_inner();
            let arg = process_expr(pairs.next().unwrap())?;
            let ret = process_expr(pairs.next().unwrap())?;
            Expr::Proc(vec![arg], Box::new(ret))
        }

        Rule::try_catch => {
            let mut pairs = pair.into_inner();
            let t = process_expr(pairs.next().unwrap())?;
            let c = process_expr(pairs.next().unwrap())?;
            Expr::Try(Box::new(t), Box::new(c))
        }

        Rule::exception => {
            let mut pairs = pair.into_inner();
            let e = process_expr(pairs.next().unwrap())?;
            Expr::Raise(Box::new(e))
        }

        Rule::none => Expr::None,

        _ => {
            eprintln!("unexpected: {:?}", pair);
            unimplemented!()
        }
    })
}