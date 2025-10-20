string_pool = static_library(
   'string_pool',     
   ['string_pool.c'], 
   c_args : c_args
)

executable('farol',
           'main.c',
           link_with : string_pool,
          c_args : c_args)

# install_targets : [farol]
# install_headers : files('string_pool.h'), install_dir : install_dirs['includedir']

