use crate::{Runtime, Handler, ExitFatal};

/// Interrupt resolution.
pub enum Resolve<'a, H: Handler> {
	/// Create interrupt resolution.
	Create(H::CreateInterrupt, ResolveCreate<'a>),
	/// Call interrupt resolution.
	Call(H::CallInterrupt, ResolveCall<'a>),
}

/// Create interrupt resolution.
pub struct ResolveCreate<'a> {
	runtime: &'a mut Runtime,
}

impl<'a> ResolveCreate<'a> {
	pub(crate) fn new(runtime: &'a mut Runtime) -> Self {
		Self { runtime }
	}
}

impl<'a> Drop for ResolveCreate<'a> {
	fn drop(&mut self) {
		self.runtime.status = Err(ExitFatal::UnhandledInterrupt.into());
		self.runtime.machine.exit(ExitFatal::UnhandledInterrupt.into());
	}
}

/// Call interrupt resolution.
pub struct ResolveCall<'a> {
	runtime: &'a mut Runtime,
}

impl<'a> ResolveCall<'a> {
	pub(crate) fn new(runtime: &'a mut Runtime) -> Self {
		Self { runtime }
	}
}

impl<'a> Drop for ResolveCall<'a> {
	fn drop(&mut self) {
		self.runtime.status = Err(ExitFatal::UnhandledInterrupt.into());
		self.runtime.machine.exit(ExitFatal::UnhandledInterrupt.into());
	}
}
