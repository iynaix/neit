use crate::{err_system::err_types::ErrTypes, tok_system::tokens::Token};
use crate::parse_systems::{Variables, COLLECTED_VARS};
use super::{AST, COLLECTED_ERRORS, LINE};

pub fn parse2(
    token: &Token,
    token_iter: &mut std::iter::Peekable<std::slice::Iter<'_, Token>>,
    ast: &mut Vec<AST>
) {
    match token {
        Token::Iden(ref id) if id == "may" => {
            while let Some(Token::Space) = token_iter.peek() {
                token_iter.next();
            }
            let var_name = match token_iter.next() {
                Some(Token::Iden(name)) => name.clone(),
                _ => {
                    if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                        unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
                    }
                    return;
                }
            };
            if COLLECTED_VARS.lock().unwrap().iter().any(|(name, _)| name == &var_name) {
                if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                    unsafe { errors.push(ErrTypes::VarAlreadyExists(LINE)) };
                }
                return;
            }
            while let Some(Token::Space) = token_iter.peek() {
                token_iter.next();
            }
            let mut found_eq = false;
            let mut found_math_op = false;
            let mut math_operator: Option<char> = None;
            if let Some(first) = token_iter.next() {
                match first {
                    Token::EqSign => { found_eq = true; },
                    Token::ADDOP => { found_math_op = true; math_operator = Some('+'); },
                    Token::SUBOP => { found_math_op = true; math_operator = Some('-'); },
                    Token::MULTIOP => { found_math_op = true; math_operator = Some('*'); },
                    Token::DIVOP => { found_math_op = true; math_operator = Some('/'); },
                    _ => {
                        if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                            unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
                        }
                        return;
                    }
                }
            } else {
                if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                    unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
                }
                return;
            }
            while let Some(Token::Space) = token_iter.peek() {
                token_iter.next();
            }
            if found_eq && !found_math_op {
                if let Some(next_tok) = token_iter.peek() {
                    match next_tok {
                        Token::ADDOP => { found_math_op = true; math_operator = Some('+'); token_iter.next(); },
                        Token::SUBOP => { found_math_op = true; math_operator = Some('-'); token_iter.next(); },
                        Token::MULTIOP => { found_math_op = true; math_operator = Some('*'); token_iter.next(); },
                        Token::DIVOP => { found_math_op = true; math_operator = Some('/'); token_iter.next(); },
                        _ => {}
                    }
                }
            } else if found_math_op && !found_eq {
                while let Some(Token::Space) = token_iter.peek() {
                    token_iter.next();
                }
                if let Some(next_tok) = token_iter.next() {
                    if let Token::EqSign = next_tok {
                        found_eq = true;
                    } else {
                        if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                            unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
                        }
                       	return;
                    }
                } else {
                    if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                        unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
                    }
                    return;
                }
            }
            while let Some(Token::Space) = token_iter.peek() {
                token_iter.next();
            }
            let raw_value = match token_iter.next() {
                Some(Token::Iden(val)) => val.clone(),
                _ => {
                    if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                        unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
                    }
                    return;
                }
            };
            if (found_eq && found_math_op) || raw_value.contains('+') || raw_value.contains('-') || raw_value.contains('*') || raw_value.contains('/') {
                let op = math_operator.unwrap_or('+');
                let math_expr = if let Some(existing) = COLLECTED_VARS.lock().unwrap().iter().find(|(name, _)| name == &var_name) {
                    format!("{}{}{}", existing.1, op, raw_value)
                } else {
                    format!("0{}{}", op, raw_value)
                };
                let var = Variables::MATH(var_name.clone(), math_expr);
                if COLLECTED_VARS.lock().unwrap().iter().all(|(name, _)| name != &var_name) {
                    COLLECTED_VARS.lock().unwrap().push((var_name.clone(), "f32"));
                }
                ast.push(AST::Var(var));
            } else {
                let var_value = if raw_value.starts_with('\'') && raw_value.ends_with('\'') && raw_value.len() >= 2 {
                    raw_value[1..raw_value.len()-1].to_string()
                } else {
                    raw_value
                };
                let var_name_static = Box::leak(var_name.clone().into_boxed_str());
                let var_type: &'static str;
                let var = if let Ok(val) = var_value.parse::<i8>() {
                    var_type = "i8";
                    Variables::I8(var_name_static, val)
                } else if let Ok(val) = var_value.parse::<i16>() {
                    var_type = "i16";
                    Variables::I16(var_name_static, val)
                } else if let Ok(val) = var_value.parse::<i32>() {
                    var_type = "i32";
                    Variables::I32(var_name_static, val)
                } else if let Ok(val) = var_value.parse::<i64>() {
                    var_type = "i64";
                    Variables::I64(var_name_static, val)
                } else if let Ok(val) = var_value.parse::<f32>() {
                    var_type = "f32";
                    Variables::F32(var_name_static, val)
                } else if let Ok(val) = var_value.parse::<f64>() {
                    var_type = "f64";
                    Variables::F64(var_name_static, val)
                } else if COLLECTED_VARS.lock().unwrap().iter().any(|(name, _)| name == &var_value) {
                    var_type = "ch";
                    Variables::REF(var_name_static, var_value)
                } else if var_value.len() == 1 {
                    var_type = "ch";
                    Variables::Char(var_name_static, var_value.chars().next().unwrap())
                } else {
                    if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                        unsafe { errors.push(ErrTypes::CharVarLen(LINE)) };
                    }
                    return;
                };
                COLLECTED_VARS.lock().unwrap().push((var_name, var_type));
                ast.push(AST::Var(var));
            }
        },
        _ => {
            if let Ok(mut errors) = COLLECTED_ERRORS.lock() {
                unsafe { errors.push(ErrTypes::UnknownCMD(LINE)) };
            }
        }
    }
}
