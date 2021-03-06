#[macro_use]
mod macros;

use self::macros::MetricFromOpts;
use crate::db::Pool;
use crate::BuildQueue;
use failure::Error;
use prometheus::proto::MetricFamily;

load_metric_type!(IntGauge as single);
load_metric_type!(IntCounter as single);
load_metric_type!(IntCounterVec as vec);
load_metric_type!(HistogramVec as vec);

metrics! {
    pub struct Metrics {
        /// Number of crates in the build queue
        queued_crates_count: IntGauge,
        /// Number of crates in the build queue that have a positive priority
        prioritized_crates_count: IntGauge,
        /// Number of crates that failed to build
        failed_crates_count: IntGauge,

        /// The number of idle database connections
        idle_db_connections: IntGauge,
        /// The number of used database connections
        used_db_connections: IntGauge,
        /// The maximum number of database connections
        max_db_connections: IntGauge,
        /// Number of attempted and failed connections to the database
        pub(crate) failed_db_connections: IntCounter,

        /// The number of currently opened file descriptors
        #[cfg(target_os = "linux")]
        open_file_descriptors: IntGauge,
        /// The number of threads being used by docs.rs
        #[cfg(target_os = "linux")]
        running_threads: IntGauge,

        /// The traffic of various docs.rs routes
        pub(crate) routes_visited: IntCounterVec["route"],
        /// The response times of various docs.rs routes
        pub(crate) response_time: HistogramVec["route"],
        /// The time it takes to render a rustdoc page
        pub(crate) rustdoc_rendering_times: HistogramVec["step"],

        /// Number of crates built
        pub(crate) total_builds: IntCounter,
        /// Number of builds that successfully generated docs
        pub(crate) successful_builds: IntCounter,
        /// Number of builds that generated a compiler error
        pub(crate) failed_builds: IntCounter,
        /// Number of builds that did not complete due to not being a library
        pub(crate) non_library_builds: IntCounter,

        /// Number of files uploaded to the storage backend
        pub(crate) uploaded_files_total: IntCounter,

        /// The number of attempted files that failed due to a memory limit
        pub(crate) html_rewrite_ooms: IntCounter,
    }

    // The Rust prometheus library treats the namespace as the "prefix" of the metric name: a
    // metric named `foo` with a prefix of `docsrs` will expose a metric called `docsrs_foo`.
    //
    // https://docs.rs/prometheus/0.9.0/prometheus/struct.Opts.html#structfield.namespace
    namespace: "docsrs",
}

impl Metrics {
    pub(crate) fn gather(
        &self,
        pool: &Pool,
        queue: &BuildQueue,
    ) -> Result<Vec<MetricFamily>, Error> {
        self.idle_db_connections.set(pool.idle_connections() as i64);
        self.used_db_connections.set(pool.used_connections() as i64);
        self.max_db_connections.set(pool.max_size() as i64);

        self.queued_crates_count.set(queue.pending_count()? as i64);
        self.prioritized_crates_count
            .set(queue.prioritized_count()? as i64);
        self.failed_crates_count.set(queue.failed_count()? as i64);

        self.gather_system_performance();
        Ok(self.registry.gather())
    }

    #[cfg(not(target_os = "linux"))]
    fn gather_system_performance(&self) {}

    #[cfg(target_os = "linux")]
    fn gather_system_performance(&self) {
        use procfs::process::Process;

        let process = Process::myself().unwrap();
        self.open_file_descriptors
            .set(process.fd().unwrap().len() as i64);
        self.running_threads
            .set(process.stat().unwrap().num_threads as i64);
    }
}
