#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# This script plots an AST file into a 2D plane. AST files are expected to be in
# the following format
#
# Row\tCol\tN\tI\tJ
# 0\t0\t2\t1\t0
# 0\t0\t2\t0\t1
# 0\t1\t2\t1\t0
# 0\t1\t2\t0\t1
# 1\t0\t2\t1\t0
# 1\t0\t2\t0\t1
#
# where each line represents an AST.
#
# The first line is the header and the following lines are the ASTs. Each AST is
# defined by the following parameters:
#
# - Row: The row of the first point of the AST
# - Col: The column of the first point of the AST
# - N: The number of points of the AST
# - I: The row increment between points of the AST
# - J: The column increment between points of the AST
#
# An AST file is generated using the following syntax:
#
# matrix_rs patterns.txt matrix.mtx --print-ast-list > ast_file.txt
#
# where `matrix_rs` is the path to the matrix_rs executable, `patterns.txt` is
# the path to the patterns file, `matrix.mtx` is the path to the matrix file and
# `ast_file.txt` is the path to the AST file to be generated.
#
# The script is then executed using the following syntax:
#
# python3 plot_ast_2d.py ast_file.txt [-o output.pdf]
#
# where `ast_file.txt` is the path to the AST file to be plotted and
# `output.pdf` is the path to the output PDF file.
#
# If the output file already exists, it will be overwritten.
#
# If the output file is not specified, the AST file will be saved in your
# current working directory with the same name as the AST file but with the
# `.pdf` extension.
#
# The script will create a temporary directory in your system's temporary
# directory. This directory will be deleted when the script finishes.

__version__ = 'v0.0.0'

__email__ = 'i.amatria@udc.es'
__author__ = 'Iñaki Amatria-Barral'

__license__ = 'I addere to any license you want to use this code under'

import os
import PyPDF2
import argparse
import tempfile
import charset_normalizer

import numpy as np
import matplotlib.pyplot as plt

from tqdm import tqdm

from concurrent.futures import ProcessPoolExecutor

def _is_int(s):
    try:
        int(s)
        return True
    except ValueError:
        return False

def read_ast_file(ast_file):
    ast_magic_header = 'Row\tCol\tN\tI\tJ'

    if not os.path.exists(ast_file):
        raise FileNotFoundError(f'AST file `{ast_file}` does not exist')

    with open(ast_file, 'r') as f:
        results = str(charset_normalizer.from_path(ast_file).best())
        asts = [line.strip() for line in results.split('\n')]
    asts = [ast for ast in asts if ast != '']

    if not asts:
        raise ValueError(f'AST file `{ast_file}` is empty')
    if asts[0] != ast_magic_header:
        raise ValueError(f'AST file `{ast_file}` is not a valid AST file')
    asts = asts[1:]
    for ast in asts:
        ast = ast.split('\t')
        if len(ast) != 5 or any([not _is_int(ast[i]) for i in range(5)]):
            raise ValueError(f'AST file `{ast_file}` is not a valid AST file')

    return [[int(x) for x in ast.split('\t')] for ast in asts]

def _is_in_matrix_block(i, j, stride_i, stride_j, ast):
    row, col, n, ii, jj = ast

    for k in range(n):
        if row + k * ii < j or row + k * ii >= j + stride_j:
            continue
        if col + k * jj < i or col + k * jj >= i + stride_i:
            continue
        return True

    return False

def _plot_matrix_block(i, j, stride_i, stride_j, asts, tmp_dir):
    colors = plt.cm.nipy_spectral(np.linspace(0, 1, len(asts)))
    ast_type_color = {(n, i, j): 0 for _, _, n, i, j in asts}
    ast_type_color = {k: colors[i] for i, k in enumerate(ast_type_color.keys())}

    fig = plt.figure()
    ax = fig.add_subplot(111)

    points_in_canvas = 0
    for ast in asts:
        if not _is_in_matrix_block(i, j, stride_i, stride_j, ast):
            continue
        points_in_canvas += 1

        row, col, n, ii, jj = ast
        for k in range(n):
            ax.add_patch(
                plt.Polygon(
                    [
                        (col + k * jj, row + k * ii),
                        (col + k * jj + 1, row + k * ii),
                        (col + k * jj + 1, row + k * ii + 1),
                        (col + k * jj, row + k * ii + 1),
                    ],
                    facecolor=ast_type_color[(n, ii, jj)],
                    alpha=0.5
                )
            )
        for k in range(n):
            if k == n - 1 and n > 1:
                break
            ax.scatter(
                col + k * jj + 0.5,
                row + k * ii + 0.5,
                color=ast_type_color[(n, ii, jj)],
                marker='o'
            )
            if n > 1:
                ax.scatter(
                    col + (k + 1) * jj + 0.5,
                    row + (k + 1) * ii + 0.5,
                    color=ast_type_color[(n, ii, jj)],
                    marker='o'
                )
                ax.add_line(
                    plt.Line2D(
                        (col + k * jj + 0.5, col + (k + 1) * jj + 0.5),
                        (row + k * ii + 0.5, row + (k + 1) * ii + 0.5),
                        color=ast_type_color[(n, ii, jj)],
                        linestyle='-'
                    )
                )

    if points_in_canvas == 0 and not (i == 0 and j == 0):
        plt.close()
        return

    plt.gca().set_axis_off()
    plt.subplots_adjust(
        top=1,
        bottom=0,
        right=1,
        left=0,
        hspace=0,
        wspace=0
    )
    plt.margins(0, 0)

    ax.set_xticks([])
    ax.set_yticks([])

    ax.set_aspect('equal')

    ax.set_xlim([i, i + stride_i])
    ax.set_ylim([j, j + stride_j])
    ax.invert_yaxis()

    save_path = os.path.join(tmp_dir, f'{i}_{j}.pdf')
    plt.savefig(save_path, bbox_inches='tight', pad_inches=0)

    plt.close()

def _merge_matrix_block(
        j,
        stride_i,
        max_row,
        width,
        height,
        target_width,
        tmp_dir
    ):
    blank_row = PyPDF2.PageObject.create_blank_page(
        width=target_width,
        height=height
    )

    for i in range(0, max_row, stride_i):
        load_path = os.path.join(tmp_dir, f'{i}_{j}.pdf')
        if not os.path.exists(load_path):
            continue

        pdf = PyPDF2.PdfReader(load_path)

        x_offset = int(i / stride_i) * width
        y_offset = 0

        page = pdf.pages[0]
        page.add_transformation(
            PyPDF2.Transformation().translate(x_offset, y_offset)
        )
        page.mediabox = blank_row.mediabox

        blank_row.merge_page(page)

    pdf_writer = PyPDF2.PdfWriter()
    pdf_writer.add_page(blank_row)
    pdf_writer.write(os.path.join(tmp_dir, f'row_{j}.pdf'))

def _log_reduction(
        k,
        reductions,
        j,
        stride_j,
        height,
        target_width,
        target_height,
        tmp_dir,
        ast_file_name
    ):
    top_idx, bot_idx = j, j + (2 ** k) * stride_j

    merge_path = os.path.join(tmp_dir, f'row_{k - 1}_')
    if k == 0:
        merge_path = os.path.join(tmp_dir, f'row_')

    top_path = merge_path + f'{top_idx}.pdf'
    bot_path = merge_path + f'{bot_idx}.pdf'

    target_page = PyPDF2.PageObject.create_blank_page(
        width=target_width,
        height=height * 2 * (2 ** k)
    )

    if os.path.exists(top_path):
        top_row = PyPDF2.PdfReader(top_path)

        top_page = top_row.pages[0]
        top_page.add_transformation(
            PyPDF2.Transformation().translate(0, height * (2 ** k))
        )
        top_page.mediabox = target_page.mediabox

        target_page.merge_page(top_page)

    if os.path.exists(bot_path):
        bot_row = PyPDF2.PdfReader(bot_path)

        bot_page = bot_row.pages[0]
        bot_page.add_transformation(
            PyPDF2.Transformation().translate(0, 0)
        )
        bot_page.mediabox = target_page.mediabox

        target_page.merge_page(bot_page)

    if k != reductions - 1:
        pdf_writer = PyPDF2.PdfWriter()
        pdf_writer.add_page(target_page)
        pdf_writer.write(os.path.join(tmp_dir, f'row_{k}_{j}.pdf'))
        return

    blank_page = PyPDF2.PageObject.create_blank_page(
        width=target_width,
        height=target_height
    )

    target_page.add_transformation(
        PyPDF2.Transformation().translate(
            0,
            target_height - target_page.mediabox.height
        )
    )
    target_page.mediabox = blank_page.mediabox

    blank_page.merge_page(target_page)

    if os.path.dirname(ast_file_name) != '':
        os.makedirs(
            os.path.dirname(ast_file_name),
            exist_ok=True
        )

    pdf_writer = PyPDF2.PdfWriter()
    pdf_writer.add_page(blank_page)
    pdf_writer.write(ast_file_name)

def print_asts_2d(asts, ast_file_name):
    stride_i = 25
    stride_j = 25

    with tempfile.TemporaryDirectory() as tmp_dir:
        max_row_idx = max([ast[1] + (ast[2] - 1) * ast[4] for ast in asts])
        max_col_idx = max([ast[0] + (ast[2] - 1) * ast[3] for ast in asts])

        max_row = stride_i * int((max_row_idx + stride_i) / stride_i)
        max_col = stride_j * int((max_col_idx + stride_j) / stride_j)

        print("Plotting PDFs...")
        total_pdfs = int((max_row / stride_i) * (max_col / stride_j))
        with tqdm(total=total_pdfs) as pbar:
            with ProcessPoolExecutor() as executor:
                for i in range(0, max_row, stride_i):
                    for j in range(0, max_col, stride_j):
                        executor.submit(
                            _plot_matrix_block,
                            i,
                            j,
                            stride_i,
                            stride_j,
                            asts,
                            tmp_dir
                        ).add_done_callback(lambda _: pbar.update())

        load_path = os.path.join(tmp_dir, '0_0.pdf')

        pdf = PyPDF2.PdfReader(load_path)
        page = pdf.pages[0]

        width = page.mediabox.width
        height = page.mediabox.height
        target_width = int(width * (max_row_idx + 1) / stride_i)
        target_height = int(height * (max_col_idx + 1) / stride_j)

        print("Merging PDFs...")
        merges = int(max_col / stride_j)
        reductions = int(np.log2(max_col / stride_j)) + 1
        with tqdm(total=merges + reductions) as pbar:
            with ProcessPoolExecutor() as executor:
                for j in range(0, max_col, stride_j):
                    executor.submit(
                        _merge_matrix_block,
                        j,
                        stride_i,
                        max_row,
                        width,
                        height,
                        target_width,
                        tmp_dir
                    ).add_done_callback(lambda _: pbar.update())

            with ProcessPoolExecutor() as executor:
                for k in range(0, reductions):
                    futures = []
                    for j in range(0, max_col, stride_j * (2 ** (k + 1))):
                        future = executor.submit(
                            _log_reduction,
                            k,
                            reductions,
                            j,
                            stride_j,
                            height,
                            target_width,
                            target_height,
                            tmp_dir,
                            ast_file_name
                        )
                        futures.append(future)

                    for future in futures:
                        future.result()

                    pbar.update()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        prog='AST 2D plotter',
        description='''A small Python utility to plot a list of ASTs in a 2D
 plane'''
    )

    parser.add_argument(
        'input_ast_file',
        type=str,
        help='the AST file to plot'
    )
    parser.add_argument(
        '-o',
        '--output-pdf',
        type=str,
        required=False,
        metavar='output_pdf',
        help='the output PDF file'
    )
    parser.add_argument(
        '-v',
        '--version',
        action='version',
        version=f'%(prog)s {__version__}'
    )

    args = parser.parse_args()

    ast_file = args.input_ast_file
    asts = read_ast_file(ast_file)

    ast_file_name = os.path.basename(ast_file)
    ast_file_name = f'{os.path.splitext(ast_file_name)[0]}.pdf'

    if args.output_pdf:
        ast_file_name = args.output_pdf

    print_asts_2d(asts, ast_file_name)
