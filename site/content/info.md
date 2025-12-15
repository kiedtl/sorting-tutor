+++
title = "Info"
+++

## Info

### Properties

Listing of various important properties of sorting algorithms.

<style>
ind:before {
  box-sizing: border-box;
  display: inline-block;
  width: 4rem;
  text-align: center;
  padding: 0 0.2rem 0 0.2rem;
  border-radius: 0.5rem;
  color: #fff;
  font-family: monospace;
  font-size: 0.7rem;
}

ind[stable]:before {
  content: 'stable';
  background: #125999;
}

ind[unstable]:before {
  content: 'unstable';
  color: #591902;
  background: #99591244;
}

ind[inplace]:before {
  content: 'inplace';
  background: #5912a9;
}

ind[outplace]:before {
  content: 'outplace';
  color: #890239;
  background: #e9529944;
}

ind[adaptive]:before {
  content: 'adaptive';
  background: #127971;
}

math {
  font-size: 0.85rem;
}
</style>

| Algorithm     | Worst                  | Best                  | Average               | Properties |
|:--------------|:-----------------------|:----------------------|:----------------------|:-----------|
| Bubble        | {{ bigo(s="n2") }}     | {{ bigo(s="n")     }} | {{ bigo(s="n2")    }} | <ind stable />   <ind inplace />  <ind adaptive /> |
| Shell         | {{ bigo(s="nlog2n") }} | {{ bigo(s="nlogn") }} | (varies)              | <ind unstable /> <ind inplace />  <ind adaptive /> |
| Selection     | {{ bigo(s="n2")    }}  | {{ bigo(s="n2")    }} | {{ bigo(s="n2")    }} | <ind unstable /> <ind inplace />  |
| Insertion     | {{ bigo(s="n2")    }}  | {{ bigo(s="n")     }} | {{ bigo(s="n2")    }} | <ind stable />   <ind inplace />  <ind adaptive /> |
| Heap          | {{ bigo(s="nlogn") }}  | {{ bigo(s="nlogn") }} | {{ bigo(s="nlogn") }} | <ind unstable /> <ind inplace />  |
| Merge         | {{ bigo(s="nlogn") }}  | {{ bigo(s="nlogn") }} | {{ bigo(s="nlogn") }} | <ind stable />   <ind outplace /> |
| Quick         | {{ bigo(s="n2")    }}  | {{ bigo(s="nlogn") }} | {{ bigo(s="nlogn") }} | <ind unstable /> <ind inplace />  |
| Timsort       | {{ bigo(s="nlogn") }}  | {{ bigo(s="n")     }} | {{ bigo(s="nlogn") }} | <ind stable />   <ind outplace /> <ind adaptive /> |
| Driftsort     | {{ bigo(s="nlogn") }}  | {{ bigo(s="n")     }} | {{ bigo(s="nlogk") }} {{ fn(n=1) }} | <ind stable />   <ind outplace /> |
| Bucket sort   | {{ bigo(s="n2")    }}  | {{ bigo(s="n+N")   }} {{ fn(n=2) }} |                       | <ind stable />   <ind outplace /> |
| Radix sort    | {{ bigo(s="dn")    }}  | {{ bigo(s="dn")    }} | {{ bigo(s="dn")    }} | <ind stable />   <ind outplace /> |

{{ fnsecstart() }}
{% fncont(n=1) %} For `k` unique items. {% end %}
{% fncont(n=2) %} For `N` buckets. {% end %}
{{ fnsecend() }}

Note that many variants of the above algorithms exist that add or remove
properties.

For example:
- Adaptive versions of Merge, Heap, and Quicksort exist.
- Mergesort *can* be done in place, but with a performance penalty.

#### Explanation of properties

<ind inplace />: the sorting algorithm doesn't need to copy the data into another
data structure to sort it.

<ind outplace />: "out-of-place"; a significant portion of the data is
duplicated or copied elsewhere while being sorted.

<ind stable />: if two elements are considered equal, the algorithm won't switch
them around. 

{% sidenote() %}
Why would we want elements that are equal, but preserve their relative order?
This doesn't make sense in the case of numbers, but think of the following data
structure:

```
struct Person {
  string name
  int age
}
```

Two `Person`s are equal if their names are equal, so the age isn't taken into
account. In that case, we could have two different `Person`s with the same name
but different ages, in which case we might want to keep their order relative to
each other.

In plain English, if we have two `Person`s in an array:

```
Person { "Zach", 33 }
Person { "Alex", 14 }
Person { "Sonia", 68 }
Person { "Sonia", 24 }
Person { "Jeffrey", 38 }
```

We would want the end array to look like:

```
Person { "Alex", 14 }
Person { "Jeffrey", 38 }
Person { "Sonia", 68 }
Person { "Sonia", 24 }
Person { "Zach", 33 }
```

with the two `Sonia`s not being switched around. This becomes important if we
sort data twice, once by one characteristic, and once by another — it wouldn't
work if the second sorting destroyed all the work done by the first sorting.
{% end %}

<ind adaptive />: if the data is partially sorted, the algorithm can take advantage
of this and do less work. A non-adaptive algorithm will do roughly the same
amount of work for random data and for completely sorted data.

### History

A brief reference of sorting algorithms and their history or origin is
summarized below, mostly gathered from Wikipedia. Donald Knuth's _The Art of
Computer Programming_, Vol. 3, Section 5.5 was also an invaluable resource.

| Algorithm   | Inventor          | Year     | Source          | Contributors |
|:------------|:------------------|:---------|:----------------|:-------------|
| Merge       | John von Neumann  | 1945 | Handwritten note, [discussed](https://doi.org/10.1145/356580.356581) by D. Knuth | |
| Radix sort  | Harold Seward     | 1954{{ fn(n=3) }} | Seward's thesis{{ fn(n=4) }} | |
| Bubble      | Edward Friend     | 1956 | [Journal of ACM Vol 3 Iss 3](https://dl.acm.org/doi/10.1145/320831.320833) | Kenneth Iverson <br /> (coined name) |
| Shellsort   | Donald Shell      | 1959 | [Communications of the ACM Vol 2 Iss 7](https://doi.org/10.1145/368370.368387) | |
| Quick       | Tony Hoare        | 1959 | [Computer Journal Vol 5, Iss 1](https://doi.org/10.1093/comjnl/5.1.10) | John Bentley <br /> Doug McIlroy <br /> (better pivot heuristics) |
| Heap        | J. W. J. Williams | 1964 | [Communications of the ACM Vol 7 Iss 6](https://doi.org/10.1145/512274.512284) | Robert Floyd <br /> (in-place variant) |
| Timsort     | Tim Peters        | 2002 | [Python-Dev ML](https://mail.python.org/pipermail/python-dev/2002-July/026837.html) | |
| Driftsort   | Lukas Bergdoll, Orson Peters | 2024 | [GitHub](https://github.com/Voultapher/sort-research-rs/blob/main/writeup/driftsort_introduction/text.md) |

Most of the above papers are paywalled, but there are, uh, "[alternative
methods](https://en.wikipedia.org/wiki/Online_piracy)" that work very well. Or,
if you're a university student, your institution probably gives you free (and
legal) access also.

<br>
<hr class="bhr">

{{ fnsecstart() }}

{% fncont(n=3) %} Radix sort appears in non-computational contexts as far back as as the
1880's, notably in machines used by the US Census Bureau (designed by Herman
Hollerith). The name and date above references the first efficient _software_
algorithm for Radix sort.
{% end %}

{% fncont(n=4) %} Seward's 1954 paper "Information Sorting in the Application of Electronic
Digital Computers to Business Operations" is seemingly [available in the MIT
archives](https://archivesspace.mit.edu/repositories/2/archival_objects/463314),
but I was not able to access it.
{% end %}

{{ fnsecend() }}
