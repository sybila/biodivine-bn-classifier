use crate::bdt::{Bdt, BdtNode, BdtNodeId};
use crate::util::Functional;
use std::io::Write;

/// Export to .dot format.
impl Bdt {
    /// Convert this tree to a .dot graph string.
    pub fn to_dot(&self) -> String {
        Vec::<u8>::new()
            .apply(|buf| self.write_dot(buf).unwrap())
            .and_then(|buf| String::from_utf8(buf).unwrap())
    }

    /// Write the .dot graph to a generic `std::io::Write`.
    pub fn write_dot(&self, out: &mut dyn Write) -> Result<(), std::io::Error> {
        writeln!(out, "digraph G {{")?;
        writeln!(out, "init__ [label=\"\", style=invis, height=0, width=0];")?;
        writeln!(out, "init__ -> 0;")?;
        if let Some(root) = self.root_id() {
            self.format_dot_recursive(out, root)?;
        }
        writeln!(out, "}}")?;
        Ok(())
    }

    /// **(internal)** Recursively move through the tree and dump all nodes.
    fn format_dot_recursive(
        &self,
        out: &mut dyn Write,
        node: BdtNodeId,
    ) -> Result<(), std::io::Error> {
        match &self[node] {
            BdtNode::Leaf { class, params } => {
                let class = format!("{class}").replace('\"', "");
                writeln!(
                    out,
                    "{}[label=\"{}({})\"];",
                    node,
                    class,
                    params.approx_cardinality()
                )?;
            }
            BdtNode::Unprocessed { classes } => {
                let classes: Vec<String> = classes
                    .iter()
                    .map(|(c, p)| format!("({c},{})", p.approx_cardinality()).replace('\"', ""))
                    .collect();
                let classes = format!("{classes:?}").replace('\"', "");
                writeln!(out, "{node}[label=\"Unprocessed({classes})\"]")?;
            }
            BdtNode::Decision {
                attribute,
                left,
                right,
                ..
            } => {
                writeln!(out, "{node}[label=\"{}\"]", self[*attribute].name)?;
                writeln!(out, "{node} -> {left} [style=dotted];")?;
                writeln!(out, "{node} -> {right} [style=filled];")?;
                self.format_dot_recursive(out, *left)?;
                self.format_dot_recursive(out, *right)?;
            }
        }
        Ok(())
    }
}
