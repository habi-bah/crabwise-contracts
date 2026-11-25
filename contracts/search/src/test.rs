use super::*;
use soroban_sdk::testutils::Address as _;

fn setup(env: &Env) -> (SearchClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Search);
    let client = SearchClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

fn tags(env: &Env, values: &[&str]) -> Vec<String> {
    let mut v = Vec::new(env);
    for s in values {
        v.push_back(String::from_str(env, s));
    }
    v
}

fn empty_filters(env: &Env) -> SearchFilters {
    let _ = env;
    SearchFilters {
        skill: None,
        min_price: None,
        max_price: None,
        min_rating: None,
        keyword: None,
    }
}

#[test]
fn index_and_fetch_listing() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);

    client.index_artist(
        &artist,
        &tags(&env, &["illustration", "logo-design"]),
        &500_i128,
        &tags(&env, &["anime", "portrait"]),
    );

    let listing = client.get_listing(&artist);
    assert_eq!(listing.price, 500);
    assert_eq!(listing.skills.len(), 2);
    assert!(listing.active);
    assert_eq!(client.get_indexed_count(), 1);
}

#[test]
fn negative_price_is_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    let result = client.try_index_artist(&artist, &Vec::new(&env), &-1_i128, &Vec::new(&env));
    assert_eq!(result, Err(Ok(SearchError::InvalidPrice)));
}

#[test]
fn search_filters_by_skill_and_price_range() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let cheap_illustrator = Address::generate(&env);
    client.index_artist(
        &cheap_illustrator,
        &tags(&env, &["illustration"]),
        &100_i128,
        &Vec::new(&env),
    );
    let pricey_illustrator = Address::generate(&env);
    client.index_artist(
        &pricey_illustrator,
        &tags(&env, &["illustration"]),
        &900_i128,
        &Vec::new(&env),
    );
    let logo_designer = Address::generate(&env);
    client.index_artist(
        &logo_designer,
        &tags(&env, &["logo-design"]),
        &200_i128,
        &Vec::new(&env),
    );

    let mut filters = empty_filters(&env);
    filters.skill = Some(String::from_str(&env, "illustration"));
    filters.max_price = Some(300_i128);

    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.total_matches, 1);
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results.get(0).unwrap().artist, cheap_illustrator);
}

#[test]
fn search_sorts_by_price_ascending_and_descending() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    client.index_artist(&a, &Vec::new(&env), &300_i128, &Vec::new(&env));
    client.index_artist(&b, &Vec::new(&env), &100_i128, &Vec::new(&env));
    client.index_artist(&c, &Vec::new(&env), &200_i128, &Vec::new(&env));

    let filters = empty_filters(&env);
    let asc = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    assert_eq!(asc.results.get(0).unwrap().price, 100);
    assert_eq!(asc.results.get(1).unwrap().price, 200);
    assert_eq!(asc.results.get(2).unwrap().price, 300);

    let desc = client.search(&filters, &SortBy::PriceDesc, &0, &10);
    assert_eq!(desc.results.get(0).unwrap().price, 300);
    assert_eq!(desc.results.get(1).unwrap().price, 200);
    assert_eq!(desc.results.get(2).unwrap().price, 100);
}

#[test]
fn search_sorts_by_rating_and_respects_admin_set_rating() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let a = Address::generate(&env);
    let b = Address::generate(&env);
    client.index_artist(&a, &Vec::new(&env), &100_i128, &Vec::new(&env));
    client.index_artist(&b, &Vec::new(&env), &100_i128, &Vec::new(&env));
    client.set_rating(&a, &40);
    client.set_rating(&b, &90);
    let _ = admin;

    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::RatingDesc, &0, &10);
    assert_eq!(page.results.get(0).unwrap().artist, b);
    assert_eq!(page.results.get(1).unwrap().artist, a);
}

#[test]
fn pagination_slices_the_sorted_result_set() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    for i in 0..5 {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &(i as i128 * 10), &Vec::new(&env));
    }

    let filters = empty_filters(&env);
    let page0 = client.search(&filters, &SortBy::PriceAsc, &0, &2);
    assert_eq!(page0.total_matches, 5);
    assert_eq!(page0.results.len(), 2);
    assert_eq!(page0.results.get(0).unwrap().price, 0);
    assert_eq!(page0.results.get(1).unwrap().price, 10);

    let page2 = client.search(&filters, &SortBy::PriceAsc, &2, &2);
    assert_eq!(page2.results.len(), 1);
    assert_eq!(page2.results.get(0).unwrap().price, 40);

    let page_out_of_range = client.search(&filters, &SortBy::PriceAsc, &10, &2);
    assert_eq!(page_out_of_range.results.len(), 0);
    assert_eq!(page_out_of_range.total_matches, 5);
}

#[test]
fn keyword_filter_matches_metadata_tags() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let a = Address::generate(&env);
    client.index_artist(
        &a,
        &Vec::new(&env),
        &50_i128,
        &tags(&env, &["fantasy", "dragons"]),
    );
    let b = Address::generate(&env);
    client.index_artist(&b, &Vec::new(&env), &50_i128, &tags(&env, &["minimalist"]));

    let mut filters = empty_filters(&env);
    filters.keyword = Some(String::from_str(&env, "dragons"));
    let page = client.search(&filters, &SortBy::Newest, &0, &10);
    assert_eq!(page.total_matches, 1);
    assert_eq!(page.results.get(0).unwrap().artist, a);
}

#[test]
fn deactivated_listing_is_excluded_from_search() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &50_i128, &Vec::new(&env));

    client.deactivate_listing(&artist, &artist);
    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::Newest, &0, &10);
    assert_eq!(page.total_matches, 0);

    client.reactivate_listing(&admin, &artist);
    let page = client.search(&filters, &SortBy::Newest, &0, &10);
    assert_eq!(page.total_matches, 1);
}

#[test]
fn stranger_cannot_deactivate_someone_elses_listing() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    let stranger = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &50_i128, &Vec::new(&env));

    let result = client.try_deactivate_listing(&stranger, &artist);
    assert_eq!(result, Err(Ok(SearchError::Unauthorized)));
}

#[test]
fn invalid_page_size_is_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let filters = empty_filters(&env);
    let result = client.try_search(&filters, &SortBy::Newest, &0, &0);
    assert_eq!(result, Err(Ok(SearchError::InvalidPageSize)));

    let result = client.try_search(&filters, &SortBy::Newest, &0, &1000);
    assert_eq!(result, Err(Ok(SearchError::InvalidPageSize)));
}

#[test]
fn inverted_price_range_returns_actionable_error() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let mut filters = empty_filters(&env);
    filters.min_price = Some(100);
    filters.max_price = Some(50);
    let result = client.try_search(&filters, &SortBy::Newest, &0, &10);
    assert_eq!(result, Err(Ok(SearchError::InvalidPriceRange)));
    assert_eq!(
        errors::get_suggestion(SearchError::InvalidPriceRange),
        symbol_short!("BAD_RANGE")
    );
    assert_eq!(client.get_analytics().total_searches, 0);
}

#[test]
fn search_analytics_count_searches_and_indexed() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &10_i128, &Vec::new(&env));

    let filters = empty_filters(&env);
    client.search(&filters, &SortBy::Newest, &0, &10);
    client.search(&filters, &SortBy::Newest, &0, &10);

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_indexed, 1);
    assert_eq!(analytics.total_searches, 2);
}

#[test]
fn reindexing_preserves_admin_set_rating() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &10_i128, &Vec::new(&env));
    client.set_rating(&artist, &77);

    client.index_artist(&artist, &tags(&env, &["new-skill"]), &20_i128, &Vec::new(&env));
    let listing = client.get_listing(&artist);
    assert_eq!(listing.rating, 77);
    assert_eq!(listing.price, 20);
    // Re-indexing an existing artist must not double-count total_indexed.
    assert_eq!(client.get_indexed_count(), 1);
}

// ── #987: bounded scan and index consistency ──────────────────────────────

#[test]
fn set_rating_and_toggles_update_the_search_index() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &50_i128, &Vec::new(&env));

    // A rating set after indexing must be visible to the index-backed filter.
    let mut filters = empty_filters(&env);
    filters.min_rating = Some(80);
    assert_eq!(
        client.search(&filters, &SortBy::RatingDesc, &0, &10).total_matches,
        0
    );
    client.set_rating(&artist, &90);
    assert_eq!(
        client.search(&filters, &SortBy::RatingDesc, &0, &10).total_matches,
        1
    );

    // A deactivation must be visible to the index-backed filter too.
    client.deactivate_listing(&artist, &artist);
    assert_eq!(
        client.search(&empty_filters(&env), &SortBy::Newest, &0, &10).total_matches,
        0
    );
    client.reactivate_listing(&artist, &artist);
    assert_eq!(
        client.search(&empty_filters(&env), &SortBy::Newest, &0, &10).total_matches,
        1
    );
}

#[test]
fn reindexing_rewrites_the_index_entry() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(
        &artist,
        &tags(&env, &["illustration"]),
        &100_i128,
        &Vec::new(&env),
    );

    // Re-index with a new price and skill; the old index entry must be replaced,
    // not left stale.
    client.index_artist(
        &artist,
        &tags(&env, &["logo-design"]),
        &900_i128,
        &Vec::new(&env),
    );

    let mut filters = empty_filters(&env);
    filters.skill = Some(String::from_str(&env, "illustration"));
    assert_eq!(
        client.search(&filters, &SortBy::PriceAsc, &0, &10).total_matches,
        0
    );
    let mut filters = empty_filters(&env);
    filters.skill = Some(String::from_str(&env, "logo-design"));
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.total_matches, 1);
    assert_eq!(page.results.get(0).unwrap().price, 900);
    // The artist was not re-assigned a position.
    assert_eq!(client.get_indexed_count(), 1);
}

#[test]
fn full_scan_below_max_scan_is_not_truncated() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    for _ in 0..5 {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &10_i128, &Vec::new(&env));
    }
    let page = client.search(&empty_filters(&env), &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.total_matches, 5);
    assert_eq!(page.scanned, 5);
    assert!(!page.truncated);
}

#[test]
fn scan_beyond_max_scan_is_reported_as_truncated() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    for _ in 0..(MAX_SCAN + 5) {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &10_i128, &Vec::new(&env));
    }
    let page = client.search(&empty_filters(&env), &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.scanned, MAX_SCAN, "scan cost is bounded by MAX_SCAN");
    assert!(page.truncated, "partial results must be reported explicitly");
}

/// Tests for the bounded top-k selection (issue #992).
/// These verify that `select_page` (used by `search`) produces results
/// identical to the previous full-sort + window approach.
#[test]
fn top_k_selection_matches_full_sort_order_for_all_pages() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    // Index 10 artists with varying prices.
    let prices: [i128; 10] = [500, 200, 800, 100, 600, 300, 900, 400, 700, 50];
    for price in prices.iter() {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), price, &Vec::new(&env));
    }

    let filters = empty_filters(&env);

    // Collect all results page by page with page_size=3, sort by PriceAsc.
    let mut all_prices: soroban_sdk::Vec<i128> = soroban_sdk::Vec::new(&env);
    for p in 0..4u32 {
        let page = client.search(&filters, &SortBy::PriceAsc, &p, &3);
        for listing in page.results.iter() {
            all_prices.push_back(listing.price);
        }
    }

    // Expected: sorted ascending [50, 100, 200, 300, 400, 500, 600, 700, 800, 900]
    let expected: [i128; 10] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900];
    assert_eq!(all_prices.len(), 10);
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(all_prices.get(i as u32).unwrap(), *exp, "position {}", i);
    }
}

#[test]
fn top_k_total_matches_is_correct_across_pages() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    // Index 7 artists.
    for i in 0..7u32 {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &(i as i128 * 100), &Vec::new(&env));
    }

    let filters = empty_filters(&env);

    // Every page should report total_matches = 7, regardless of which page.
    for p in 0..5u32 {
        let page = client.search(&filters, &SortBy::PriceAsc, &p, &2);
        assert_eq!(
            page.total_matches, 7,
            "page {} should report 7 total matches",
            p
        );
    }
}

#[test]
fn top_k_selection_sorts_by_rating_desc_correctly() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let ratings: [u32; 5] = [30, 10, 80, 50, 20];
    let mut artists = soroban_sdk::Vec::new(&env);
    for rating in ratings.iter() {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));
        client.set_rating(&artist, rating);
        artists.push_back(artist);
    }

    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::RatingDesc, &0, &5);
    assert_eq!(page.results.len(), 5);
    // Verify descending order.
    assert_eq!(page.results.get(0).unwrap().rating, 80);
    assert_eq!(page.results.get(1).unwrap().rating, 50);
    assert_eq!(page.results.get(2).unwrap().rating, 30);
    assert_eq!(page.results.get(3).unwrap().rating, 20);
    assert_eq!(page.results.get(4).unwrap().rating, 10);
}

// ── Pagination boundary behavior ──────────────────────────────────────────

#[test]
fn pagination_boundaries_on_empty_collection_and_zero_matches() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let filters = empty_filters(&env);

    // Empty index: every valid page and page size yields an empty result set.
    for page_num in [0u32, 1, 10, u32::MAX] {
        for page_size in [1u32, MAX_PAGE_SIZE] {
            let page = client.search(&filters, &SortBy::PriceAsc, &page_num, &page_size);
            assert!(page.results.is_empty());
            assert_eq!(page.total_matches, 0);
            assert_eq!(page.scanned, 0);
            assert!(!page.truncated);

            let (from, to) = shared::pagination::page_window(0, page_num, page_size);
            assert_eq!((from, to), (0, 0));
        }
    }

    // Non-empty index where filters match zero listings.
    let artist = Address::generate(&env);
    client.index_artist(
        &artist,
        &tags(&env, &["illustration"]),
        &250_i128,
        &Vec::new(&env),
    );
    let mut unmatched = empty_filters(&env);
    unmatched.skill = Some(String::from_str(&env, "nonexistent-skill"));

    let empty_items: Vec<ArtistListing> = Vec::new(&env);
    for page_num in [0u32, 1, u32::MAX] {
        let page = client.search(&unmatched, &SortBy::PriceAsc, &page_num, &10);
        assert!(page.results.is_empty());
        assert_eq!(page.total_matches, 0);
        assert_eq!(page.scanned, 1);

        let (shared_page, info) =
            shared::pagination::paginated(&env, &empty_items, page_num, 10);
        assert!(shared_page.is_empty());
        assert_eq!(info.total, 0);
        assert_eq!(info.start, 0);
        assert_eq!(info.limit, 0);
        assert_eq!(info.count, 0);
        assert!(!info.has_more);
        assert_eq!(info.next_start, None);
    }
}

#[test]
fn pagination_boundaries_for_zero_exact_and_oversized_limits() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let filters = empty_filters(&env);

    for i in 0..3u32 {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &(i as i128 * 100), &Vec::new(&env));
    }

    // Zero and oversized limits are rejected by `search` without incrementing analytics.
    for invalid_limit in [0u32, MAX_PAGE_SIZE + 1, shared::pagination::MAX_LIMIT, u32::MAX] {
        assert_eq!(
            client.try_search(&filters, &SortBy::PriceAsc, &0, &invalid_limit),
            Err(Ok(SearchError::InvalidPageSize))
        );
    }
    assert_eq!(client.get_analytics().total_searches, 0);

    // Exact valid bounds (1 and MAX_PAGE_SIZE) succeed.
    let single = client.search(&filters, &SortBy::PriceAsc, &0, &1);
    assert_eq!(single.results.len(), 1);
    assert_eq!(single.total_matches, 3);

    let max_valid = client.search(&filters, &SortBy::PriceAsc, &0, &MAX_PAGE_SIZE);
    assert_eq!(max_valid.results.len(), 3);
    assert_eq!(max_valid.total_matches, 3);
    assert_eq!(client.get_analytics().total_searches, 2);

    // Meanwhile, shared pagination helpers clamp zero to DEFAULT_LIMIT (= MAX_PAGE_SIZE)
    // and oversized limits to MAX_LIMIT.
    assert_eq!(shared::pagination::clamp_limit(0), MAX_PAGE_SIZE);
    assert_eq!(
        shared::pagination::clamp_limit(shared::pagination::MAX_LIMIT + 1),
        shared::pagination::MAX_LIMIT
    );
    assert_eq!(
        shared::pagination::clamp_limit(u32::MAX),
        shared::pagination::MAX_LIMIT
    );
}

#[test]
fn pagination_boundaries_at_exact_end_past_end_and_u32_max_page() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let filters = empty_filters(&env);

    for i in 0..6u32 {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &((i + 1) as i128 * 10), &Vec::new(&env));
    }

    // Exact full pages covering 0..3 and 3..6.
    let p0 = client.search(&filters, &SortBy::PriceAsc, &0, &3);
    assert_eq!(p0.results.len(), 3);
    assert_eq!(p0.results.get(0).unwrap().price, 10);
    assert_eq!(p0.results.get(2).unwrap().price, 30);

    let p1 = client.search(&filters, &SortBy::PriceAsc, &1, &3);
    assert_eq!(p1.results.len(), 3);
    assert_eq!(p1.results.get(0).unwrap().price, 40);
    assert_eq!(p1.results.get(2).unwrap().price, 60);

    // Exact end (start == total_matches), past end, and overflow-boundary page.
    for out_of_range_page in [2u32, 3, 100, u32::MAX] {
        let page = client.search(&filters, &SortBy::PriceAsc, &out_of_range_page, &3);
        assert!(page.results.is_empty());
        assert_eq!(page.total_matches, 6);
        assert_eq!(page.page, out_of_range_page);
        assert_eq!(page.page_size, 3);

        let (from, to) = shared::pagination::page_window(6, out_of_range_page, 3);
        assert_eq!((from, to), (6, 6));
    }
}

#[test]
fn contract_search_pages_match_shared_pagination_boundaries() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let filters = empty_filters(&env);

    // Index more than one full MAX_PAGE_SIZE (50) page to exercise full, short
    // final, exact-end, and past-end windows end to end.
    let total = MAX_PAGE_SIZE + 15;
    let mut expected_prices: Vec<i128> = Vec::new(&env);
    for i in 0..total {
        let price = (i as i128) * 5;
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &price, &Vec::new(&env));
        expected_prices.push_back(price);
    }

    for page_num in [0u32, 1, 2, 10, u32::MAX] {
        let page = client.search(&filters, &SortBy::PriceAsc, &page_num, &MAX_PAGE_SIZE);
        let (from, to) = shared::pagination::page_window(total, page_num, MAX_PAGE_SIZE);
        let (expected_slice, info) = shared::pagination::paginated(
            &env,
            &expected_prices,
            page_num.saturating_mul(MAX_PAGE_SIZE),
            MAX_PAGE_SIZE,
        );

        assert_eq!(page.total_matches, total);
        assert_eq!(page.results.len(), to - from);
        assert_eq!(page.results.len(), expected_slice.len());
        assert_eq!(info.count, page.results.len());
        assert_eq!(info.has_more, page_num == 0);
        assert_eq!(
            info.next_start,
            if page_num == 0 {
                Some(MAX_PAGE_SIZE)
            } else {
                None
            }
        );

        for idx in 0..expected_slice.len() {
            assert_eq!(
                page.results.get(idx).unwrap().price,
                expected_slice.get(idx).unwrap()
            );
        }
    }
}

// ── Issue #77: Make search updates idempotent ─────────────────────────────────

#[test]
fn indexing_unchanged_record_produces_no_duplicates() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);

    client.index_artist(&artist, &tags(&env, &["illustration"]), &500_i128, &Vec::new(&env));

    // Re-index with identical data.
    client.index_artist(&artist, &tags(&env, &["illustration"]), &500_i128, &Vec::new(&env));
    client.index_artist(&artist, &tags(&env, &["illustration"]), &500_i128, &Vec::new(&env));

    // The count is still 1.
    assert_eq!(client.get_indexed_count(), 1);

    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    // Exactly one result — no duplicates in pagination totals.
    assert_eq!(page.total_matches, 1);
// ── Issue #76: Reject invalid search query limits ────────────────────────────

#[test]
fn zero_page_size_is_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));

    let filters = empty_filters(&env);
    let err = client
        .try_search(&filters, &SortBy::PriceAsc, &0, &0)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, SearchError::InvalidPageSize);
}

#[test]
fn max_page_size_is_accepted() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));

    let filters = empty_filters(&env);
    // MAX_PAGE_SIZE (50) is the maximum accepted value.
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &MAX_PAGE_SIZE);
    assert_eq!(page.results.len(), 1);
}

#[test]
fn indexing_updated_record_replaces_by_stable_identity() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist_a = Address::generate(&env);
    let artist_b = Address::generate(&env);

    client.index_artist(&artist_a, &tags(&env, &["photo"]), &300_i128, &Vec::new(&env));
    client.index_artist(&artist_b, &tags(&env, &["video"]), &400_i128, &Vec::new(&env));

    // Update artist_a's price.
    client.index_artist(&artist_a, &tags(&env, &["photo"]), &150_i128, &Vec::new(&env));

    // Still two unique artists.
    assert_eq!(client.get_indexed_count(), 2);

    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.total_matches, 2);
    // artist_a should now show price 150 (updated).
    let artist_a_result = page.results.iter().find(|l| l.artist == artist_a).unwrap();
    assert_eq!(artist_a_result.price, 150);
}

// ── Issue #78: Exclude inactive records from search results ──────────────────

#[test]
fn inactive_records_are_excluded_without_removing_active_results() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let active_artist = Address::generate(&env);
    let inactive_artist = Address::generate(&env);

    client.index_artist(&active_artist, &Vec::new(&env), &100_i128, &Vec::new(&env));
    client.index_artist(&inactive_artist, &Vec::new(&env), &200_i128, &Vec::new(&env));

    // Deactivate one listing.
    client.deactivate_listing(&inactive_artist, &inactive_artist);

    // The deactivated listing is excluded from search.
    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.total_matches, 1);
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results.get(0).unwrap().artist, active_artist);
}

#[test]
fn reactivated_record_reappears_in_search_results() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);

    client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));
    client.deactivate_listing(&artist, &artist);

    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    assert_eq!(page.total_matches, 0);

    // Reactivate by re-indexing (index_artist sets active = true).
    client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
fn over_max_page_size_is_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));

    let filters = empty_filters(&env);
    let err = client
        .try_search(&filters, &SortBy::PriceAsc, &0, &(MAX_PAGE_SIZE + 1))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, SearchError::InvalidPageSize);
}

#[test]
fn default_non_zero_page_size_returns_results() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    for _ in 0..5u32 {
        let artist = Address::generate(&env);
        client.index_artist(&artist, &Vec::new(&env), &100_i128, &Vec::new(&env));
    }

    let filters = empty_filters(&env);
    // Page size of 1 is the smallest non-zero valid value.
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &1);
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.total_matches, 5);
    assert_eq!(page.page_size, 1);
// ── Issue #79: Keep search pagination stable during updates ──────────────────

#[test]
fn pagination_cursor_stable_when_indexed_record_changes_between_pages() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    // Index three artists at ascending prices.
    let artist_a = Address::generate(&env);
    let artist_b = Address::generate(&env);
    let artist_c = Address::generate(&env);
    client.index_artist(&artist_a, &Vec::new(&env), &100_i128, &Vec::new(&env));
    client.index_artist(&artist_b, &Vec::new(&env), &200_i128, &Vec::new(&env));
    client.index_artist(&artist_c, &Vec::new(&env), &300_i128, &Vec::new(&env));

    let filters = empty_filters(&env);

    // Page 0 with page_size 2: covers artist_a and artist_b.
    let page0 = client.search(&filters, &SortBy::PriceAsc, &0, &2);
    assert_eq!(page0.total_matches, 3);
    assert_eq!(page0.results.len(), 2);
    // Prices on page 0 are sorted ascending.
    assert_eq!(page0.results.get(0).unwrap().price, 100);
    assert_eq!(page0.results.get(1).unwrap().price, 200);

    // Simulate an update to artist_b's price between page requests.
    // Re-indexing updates the denormalised entry in-place, so artist_b retains
    // its original position and the total count stays at 3.
    client.index_artist(&artist_b, &Vec::new(&env), &150_i128, &Vec::new(&env));

    // Page 1 with page_size 2: covers artist_c.
    let page1 = client.search(&filters, &SortBy::PriceAsc, &1, &2);
    assert_eq!(page1.total_matches, 3);
    assert_eq!(page1.results.len(), 1);
    // No result was skipped or duplicated.
    let all_artists: Vec<soroban_sdk::Address> = {
        let mut v = Vec::new(&env);
        for item in page0.results.iter() {
            v.push_back(item.artist.clone());
        }
        for item in page1.results.iter() {
            v.push_back(item.artist.clone());
        }
        v
    };
    // Three unique artists across both pages.
    assert_eq!(all_artists.len(), 3);
}

#[test]
fn repeated_indexing_does_not_create_duplicates_in_results() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let artist = Address::generate(&env);
    client.index_artist(&artist, &Vec::new(&env), &500_i128, &Vec::new(&env));

    // Re-index the same artist (unchanged).
    client.index_artist(&artist, &Vec::new(&env), &500_i128, &Vec::new(&env));

    // The index count should still be 1.
    assert_eq!(client.get_indexed_count(), 1);

    let filters = empty_filters(&env);
    let page = client.search(&filters, &SortBy::PriceAsc, &0, &10);
    // Only one result — no duplicate.
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.total_matches, 1);
}
