# 作业

## [Matrix 代码的 Python 封装实现](python-binding/src/matrix.rs)

```shell
❯ rye run ipython
Python 3.12.3 (main, Apr 15 2024, 17:43:11) [Clang 17.0.6 ]
Type 'copyright', 'credits' or 'license' for more information
IPython 8.26.0 -- An enhanced Interactive Python. Type '?' for help.

In [1]: from algo import Matrix

In [2]: a = Matrix([[1,2],[3,4]]);

In [3]: b = Matrix([[5,6],[7,8]])

In [4]: a.multiply([[5,6],[7,8]])
Out[4]: Matrix( row=2, col=2, {19 22, 43 50})

In [5]: a.mul(b)
Out[5]: Matrix( row=2, col=2, {19 22, 43 50})

In [6]: a
Out[6]: Matrix( row=2, col=2, {1 2, 3 4})

In [7]: b
Out[7]: Matrix( row=2, col=2, {5 6, 7 8})

```

## [学习使用 [roaring bitmap](https://docs.rs/roaring/latest/roaring/) 使用 pyo3 为其提供接口（可以任选其一），供 python 使用](python-binding/src/roaring.rs)

```shell
❯ rye run ipython
Python 3.12.3 (main, Apr 15 2024, 17:43:11) [Clang 17.0.6 ]
Type 'copyright', 'credits' or 'license' for more information
IPython 8.26.0 -- An enhanced Interactive Python. Type '?' for help.

In [1]: from algo import Bitmap

In [2]: a = Bitmap()

In [3]: b = Bitmap()

In [4]: a.insert(1)

In [5]: a
Out[5]: RoaringBitmap<[1]>

In [6]: a.insert(2)

In [7]: a.contains(2)
Out[7]: True

In [8]: a.contains(3)
Out[8]: False

In [9]: a.len()
Out[9]: 2

In [10]: b.is_empty()
Out[10]: True

In [11]: a.is_disjoint(b)
Out[11]: True

In [12]: b.insert(3)

In [13]: a.union(b)
Out[13]: RoaringBitmap<[1, 2, 3]>

In [14]: a.difference(b)
Out[14]: RoaringBitmap<[1, 2]>

In [15]: a
Out[15]: RoaringBitmap<[1, 2]>

In [16]: a.intersection(b)
Out[16]: RoaringBitmap<[]>

In [17]: a.remove(1)

In [18]: a
Out[18]: RoaringBitmap<[2]>
```