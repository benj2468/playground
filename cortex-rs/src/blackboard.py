#!/usr/bin/env python
#
# License: BSD
#   https://raw.githubusercontent.com/splintered-reality/py_trees/devel/LICENSE
#
##############################################################################
# Documentation
##############################################################################

"""
A py_trees demo.

.. argparse::
   :module: py_trees.demos.blackboard
   :func: command_line_argument_parser
   :prog: py-trees-demo-blackboard

.. graphviz:: dot/demo-blackboard.dot
   :align: center
   :caption: Dot Graph

.. figure:: images/blackboard_demo.png
   :align: center

   Console Screenshot
"""

##############################################################################
# Imports
##############################################################################

import argparse
import sys
import typing

import py_trees
from py_trees.common import Status
import py_trees.console as console

import cortex

##############################################################################
# Classes
##############################################################################


def description() -> str:
    """
    Print description and usage information about the program.

    Returns:
       the program description string
    """
    content = "Demonstrates usage of the blackboard and related behaviours.\n"
    content += "\n"
    content += "A sequence is populated with a few behaviours that exercise\n"
    content += "reading and writing on the Blackboard in interesting ways.\n"

    if py_trees.console.has_colours:
        banner_line = console.green + "*" * 79 + "\n" + console.reset
        s = banner_line
        s += console.bold_white + "Blackboard".center(79) + "\n" + console.reset
        s += banner_line
        s += "\n"
        s += content
        s += "\n"
        s += banner_line
    else:
        s = content
    return s


def epilog() -> typing.Optional[str]:
    """
    Print a noodly epilog for --help.

    Returns:
       the noodly message
    """
    if py_trees.console.has_colours:
        return (
            console.cyan
            + "And his noodly appendage reached forth to tickle the blessed...\n"
            + console.reset
        )
    else:
        return None


def command_line_argument_parser() -> argparse.ArgumentParser:
    """
    Process command line arguments.

    Returns:
        the argument parser
    """
    parser = argparse.ArgumentParser(
        description=description(),
        epilog=epilog(),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    render_group = parser.add_mutually_exclusive_group()
    render_group.add_argument(
        "-r", "--render", action="store_true", help="render dot tree to file"
    )
    render_group.add_argument(
        "--render-with-blackboard-variables",
        action="store_true",
        help="render dot tree to file with blackboard variables",
    )
    return parser

class WriteValue(py_trees.behaviour.Behaviour):

    def __init__(self, name: str):
        super().__init__(name = name)

        self.blackboard = py_trees.blackboard.Client(name=name)
        self.blackboard.register_key(key="cortex", access=py_trees.common.Access.WRITE)

    def update(self) -> Status:
        self.blackboard.cortex.tick(200)
        
        return Status.SUCCESS




def create_root() -> py_trees.behaviour.Behaviour:
    """
    Create the root behaviour and it's subtree.

    Returns:
        the root behaviour
    """
    root = py_trees.composites.Sequence(name="Blackboard Demo", memory=True)
    set_blackboard_variable = py_trees.behaviours.SetBlackboardVariable(
        name="Set Cortex",
        variable_name="cortex",
        variable_value=cortex.Cortex(),
        overwrite=True,
    )
    check_pre_write = py_trees.behaviours.CheckBlackboardVariableValue(
        name="Check Cortex Value",
        check=py_trees.common.ComparisonExpression(
            variable="cortex.get", value=100, operator=lambda v, e: v() <= e
        ),
    )
    override_variable = WriteValue(name = "Write Cortex")
    check_post_write = py_trees.behaviours.CheckBlackboardVariableValue(
        name="Check Cortex Value",
        check=py_trees.common.ComparisonExpression(
            variable="cortex.get", value=200, operator=lambda v, e: v() >= e
        ),
    )
    root.add_children(
        [
            set_blackboard_variable,
            check_pre_write,
            override_variable,
            check_post_write
        ]
    )
    return root


##############################################################################
# Main
##############################################################################


def main() -> None:
    """Entry point for the demo script."""
    args = command_line_argument_parser().parse_args()
    print(description())
    py_trees.logging.level = py_trees.logging.Level.DEBUG
    py_trees.blackboard.Blackboard.enable_activity_stream(maximum_size=100)

    root = create_root()

    ####################
    # Rendering
    ####################
    if args.render:
        py_trees.display.render_dot_tree(root, with_blackboard_variables=False)
        sys.exit()
    if args.render_with_blackboard_variables:
        py_trees.display.render_dot_tree(root, with_blackboard_variables=True)
        sys.exit()

    ####################
    # Execute
    ####################
    root.setup_with_descendants()

    print("\n--------- Tick 0 ---------\n")
    root.tick_once()
    print("\n")
    print(py_trees.display.unicode_tree(root, show_status=True))
    print("--------------------------\n")
    print(py_trees.display.unicode_blackboard())
    print("--------------------------\n")
    print(py_trees.display.unicode_blackboard(display_only_key_metadata=True))
    print("--------------------------\n")
    print(py_trees.display.unicode_blackboard_activity_stream())


if __name__ == "__main__":
    main()