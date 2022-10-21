use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fmt::Display;
use syn::{
  Block, Expr, ExprAssign, ExprLit, ExprPath, ExprReference, ExprUnary, Ident, Lit, LitInt, Local,
  Pat, PatIdent, Stmt, UnOp,
};

pub type Place = Ident;
#[derive(Debug, Clone)]
pub enum Value {
  Unit,
  Lit(Lit),
  Ref(Place),
  Undefined,
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::Unit => write!(f, "()")?,
      Value::Lit(l) => match l {
        Lit::Int(i) => write!(f, "{}", i.token())?,
        _ => todo!("{l:?}"),
      },
      Value::Ref(p) => write!(f, "&{}", p)?,
      Value::Undefined => write!(f, "undefined")?,
    }
    Ok(())
  }
}

#[derive(Default, Debug)]
pub struct Environment(HashMap<Place, Value>);
impl Environment {
  pub fn lookup(&self, place: &Place) -> Result<&Value> {
    let value = self
      .0
      .get(place)
      .with_context(|| format!("Cannot find place: {place:?}"))?;
    match value {
      Value::Undefined => bail!("Attempting to read undefined place: {place:?}"),
      _ => Ok(value),
    }
  }

  pub fn insert(&mut self, place: Place, value: Value) {
    self.0.insert(place, value);
  }
}

impl Display for Environment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut entries = self.0.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(k, _)| k.clone());
    for (k, v) in entries {
      write!(f, "{} ↦ {}\n", k.to_string(), v)?;
    }

    Ok(())
  }
}

pub trait Interpreter {
  fn eval_block(&self, block: &Block, env: &mut Environment) -> Result<()>;

  fn interpret(&self, code: &str) -> Result<Environment> {
    let block: Block = syn::parse_str(code)?;
    let mut env = Environment::default();
    self.eval_block(&block, &mut env)?;
    Ok(env)
  }
}

pub struct ReferenceModel;
impl ReferenceModel {
  fn eval_place(&self, expr: &Expr, env: &Environment) -> Result<Place> {
    Ok(match expr {
      Expr::Path(ExprPath { path, .. }) => path.get_ident().unwrap().clone(),
      Expr::Unary(ExprUnary {
        op: UnOp::Deref(..),
        expr,
        ..
      }) => {
        let place = self.eval_place(expr, env)?;
        match env.lookup(&place)? {
          Value::Ref(place) => place.clone(),
          v => bail!("Cannot deref value: {v:?}"),
        }
      }
      _ => unimplemented!("{expr:#?}"),
    })
  }

  fn eval_expr(&self, expr: &Expr, env: &mut Environment) -> Result<Value> {
    Ok(match expr {
      Expr::Lit(ExprLit { lit, .. }) => Value::Lit(lit.clone()),
      Expr::Path(ExprPath { path, .. }) => env.lookup(path.get_ident().unwrap())?.clone(),
      Expr::Reference(ExprReference { expr: inner, .. }) => match &**inner {
        Expr::Path(ExprPath { path, .. }) => Value::Ref(path.get_ident().unwrap().clone()),
        e => unimplemented!("{e:#?}"),
      },
      Expr::Unary(ExprUnary {
        op: UnOp::Deref(_), ..
      }) => {
        let place = self.eval_place(expr, env)?;
        env.lookup(&place)?.clone()
      }
      Expr::Assign(ExprAssign { left, right, .. }) => {
        let l = self.eval_place(left, env)?;
        let r = self.eval_expr(right, env)?;
        env.insert(l, r);
        Value::Unit
      }
      e => unimplemented!("{e:#?}"),
    })
  }

  fn eval_stmt(&self, stmt: &Stmt, env: &mut Environment) -> Result<()> {
    Ok(match stmt {
      Stmt::Local(Local { pat, init, .. }) => {
        let lhs = match pat {
          Pat::Ident(PatIdent { ident, .. }) => ident,
          _ => unimplemented!(),
        };
        let v = match init.as_ref() {
          Some((_, rhs)) => self.eval_expr(&*rhs, env)?,
          None => Value::Undefined,
        };
        env.insert(lhs.clone(), v);
      }
      Stmt::Semi(expr, _) => {
        self.eval_expr(expr, env)?;
      }
      s => unimplemented!("{s:#?}"),
    })
  }
}

impl Interpreter for ReferenceModel {
  fn eval_block(&self, block: &Block, env: &mut Environment) -> Result<()> {
    for stmt in &block.stmts {
      self.eval_stmt(stmt, env)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn ok() -> Result<()> {
    let block: Block = syn::parse_quote! {{
      let a = 1;
      let mut b;
      b = &a;
      let c = *b;
    }};

    let model = ReferenceModel;
    let mut env = Environment::default();
    model.eval_block(&block, &mut env).unwrap();
    println!("{env}");

    Ok(())
  }
}
