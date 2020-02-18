/*******************************************************************************
 * Copyright (c) 2019 Fraunhofer FOKUS and others.
 *
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License 2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

/// Holds a set of native strings, in default platform encoding plus possibly 
/// intermediate representations.
/// The strings can be accessed via the `get_strings` method.
pub trait StringHolder<'o> {
    type NativeStrIter: Iterator<Item = *mut i8> + 'o;

    /// Returns an iterator over strings in the default platform encoding.
    fn get_strings(&'o self) -> Self::NativeStrIter;
}