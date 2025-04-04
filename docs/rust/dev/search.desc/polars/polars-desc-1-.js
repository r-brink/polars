searchState.loadedDescShard("polars", 1, "Unpack to <code>ChunkedArray</code> of dtype <code>[DataType::Int64]</code>\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::Int8]</code>\nConvert the values of this Series to a ListChunked with a …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCheck if Series is empty.\nCheck if numeric value is finite\nCheck if float value is infinite\nCheck if float value is NaN (note this is different than …\nCheck if float value is NaN (note this is different than …\nGet a mask of the non-null values.\nGet a mask of the null values.\niterate over <code>Series</code> as <code>AnyValue</code>.\nGet length of series.\nTake <code>num_elements</code> from the top as a zero copy view.\nUnpack to <code>ChunkedArray</code> of dtype list\nLess than comparison.\nCreate a boolean mask by checking if self &lt; rhs.\nLess than or equal comparison\nCreate a boolean mask by checking if self &lt;= rhs.\nReturns the maximum value in the array, according to the …\nGet the max of the Series as a new Series of length 1.\nReturns the mean value in the array Returns an option …\nReturns the median value in the array Returns an option …\nGet the median of the Series as a new Series of length 1.\nReturns the minimum value in the array, according to the …\nGet the min of the Series as a new Series of length 1.\nNumber of chunks in this Series\nGet unique values in the Series.\nName of series.\nConstruct a new <code>Series</code> from a collection of <code>AnyValue</code>.\nCreate a new empty Series.\nCreate a new Series filled with values from the given …\nCheck for inequality.\nCreate a boolean mask by checking for inequality.\nCheck for inequality where <code>None == None</code>.\nCreate a boolean mask by checking for inequality.\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::Null]</code>\nCount the null values.\nGet the product of an array.\nGet the quantile of the ChunkedArray as a new Series of …\nAggregate all chunks to a contiguous array of memory.\nRename the Series.\nRename series.\nreturn a Series in reversed order\nApply a custom function over a rolling/ moving window of …\nSample a fraction between 0.0-1.0 of this <code>ChunkedArray</code>.\nShift the values by a given period and fill the parts that …\nShrink the capacity of this array to fit its length.\nShrink the capacity of this array to fit its length.\nGet a zero copy view of the data.\nSort the series with specific options.\nReturns the std value in the array Returns an option …\nGet the standard deviation of the Series as a new Series …\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::String]</code>\nCast throws an error if conversion had overflows\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::Struct]</code>\nCompute the sum of all values in this Series. Returns …\nGet the sum of the Series as a new Series of length 1. …\nGet the tail of the Series.\nTake by index. This operation is clone.\nTake function that checks of null state in <code>ChunkIdx</code>.\nTake by index. This operation is clone.\nTake by index.\nTake by index if ChunkedArray contains a single chunk.\nTake by index. This operation is clone.\nTake by index.\nTake by index if ChunkedArray contains a single chunk.\nTake by index if ChunkedArray contains a single chunk.\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::Time]</code>\nConvert a chunk in the Series to the correct Arrow type. …\nCast numerical types to f64, and keep floats as is.\nCast a datelike Series to their physical representation. …\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::UInt16]</code>\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::UInt32]</code>\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::UInt64]</code>\nUnpack to <code>ChunkedArray</code> of dtype <code>[DataType::UInt8]</code>\nGet unique values in the Series.\nCompute the unique elements, but maintain order. This …\nReturns the var value in the array Returns an option …\nGet the variance of the Series as a new Series of length 1.\nReturn this Series with a new name.\nCreate a new ChunkedArray with values from self where the …\nChecked integer division. Computes self / rhs, returning …\nChecked integer division. Computes self / rhs, returning …\ndrop nulls\nignore nulls\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nA wrapper type that should make it a bit more clear that …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSwaps inner state with the <code>array</code>. Prefer …\nTemporary swaps out the array, and restores the original …")